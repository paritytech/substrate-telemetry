// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

mod aggregator;
mod feed_message;
mod feed_verifier_message;
mod find_location;
mod state;
use std::str::FromStr;
use tokio::time::{Duration, Instant};

use aggregator::{
    AggregatorOpts, AggregatorSet, FromFeedWebsocket, FromShardWebsocket, ToFeedWebsocket,
    ToShardWebsocket,
};
use bincode::Options;
use common::http_utils;
use common::internal_messages;
use common::ready_chunks_all::ReadyChunksAll;
use futures::{SinkExt, StreamExt};
use hyper::{Method, Response};
use simple_logger::SimpleLogger;
use structopt::StructOpt;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = "Substrate Telemetry Backend Core";
const ABOUT: &str = "This is the Telemetry Backend Core that receives telemetry messages \
                     from Substrate/Polkadot nodes and provides the data to a subsribed feed";

#[derive(StructOpt, Debug)]
#[structopt(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
    /// This is the socket address that Telemetry is listening to. This is restricted to
    /// localhost (127.0.0.1) by default and should be fine for most use cases. If
    /// you are using Telemetry in a container, you likely want to set this to '0.0.0.0:8000'
    #[structopt(short = "l", long = "listen", default_value = "127.0.0.1:8000")]
    socket: std::net::SocketAddr,
    /// The desired log level; one of 'error', 'warn', 'info', 'debug' or 'trace', where
    /// 'error' only logs errors and 'trace' logs everything.
    #[structopt(long = "log", default_value = "info")]
    log_level: log::LevelFilter,
    /// Space delimited list of the names of chains that are not allowed to connect to
    /// telemetry. Case sensitive.
    #[structopt(long, required = false)]
    denylist: Vec<String>,
    /// If it takes longer than this number of seconds to send the current batch of messages
    /// to a feed, the feed connection will be closed.
    #[structopt(long, default_value = "10")]
    feed_timeout: u64,
    /// Number of worker threads to spawn. If "0" is given, use the number of CPUs available
    /// on the machine. If no value is given, use an internal default that we have deemed sane.
    #[structopt(long)]
    worker_threads: Option<usize>,
    /// Each aggregator keeps track of the entire node state. Feed subscriptions are split across
    /// aggregators.
    #[structopt(long)]
    num_aggregators: Option<usize>,
    /// How big can the message queue for each aggregator grow before we start dropping non-essential
    /// messages in an attempt to let it reduce?
    #[structopt(long)]
    aggregator_queue_len: Option<usize>,
    /// How many nodes from third party chains are allowed to connect before we prevent connections from them.
    #[structopt(long, default_value = "1000")]
    max_third_party_nodes: usize,
    /// Flag to expose the IP addresses of all connected nodes to the feed subscribers.
    #[structopt(long)]
    pub expose_node_ips: bool,
}

fn main() {
    let opts = Opts::from_args();

    SimpleLogger::new()
        .with_level(opts.log_level)
        .init()
        .expect("Must be able to start a logger");

    log::info!("Starting Telemetry Core version: {}", VERSION);

    let worker_threads = match opts.worker_threads {
        Some(0) => num_cpus::get(),
        Some(n) => n,
        // By default, use a max of 8 worker threads, as perf
        // testing has found that to be a good sweet spot.
        None => usize::min(num_cpus::get(), 8),
    };

    let num_aggregators = match opts.num_aggregators {
        Some(0) => num_cpus::get(),
        Some(n) => n,
        // For now, we just have 1 aggregator loop by default,
        // but we may want to be smarter here eventually.
        None => 1,
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(worker_threads)
        .thread_name("telemetry_core_worker")
        .build()
        .unwrap()
        .block_on(async {
            if let Err(e) = start_server(num_aggregators, opts).await {
                log::error!("Error starting server: {}", e);
            }
        });
}

/// Declare our routes and start the server.
async fn start_server(num_aggregators: usize, opts: Opts) -> anyhow::Result<()> {
    let aggregator_queue_len = opts.aggregator_queue_len.unwrap_or(10_000);
    let aggregator = AggregatorSet::spawn(
        num_aggregators,
        AggregatorOpts {
            max_queue_len: aggregator_queue_len,
            denylist: opts.denylist,
            max_third_party_nodes: opts.max_third_party_nodes,
            expose_node_ips: opts.expose_node_ips,
        },
    )
    .await?;
    let socket_addr = opts.socket;
    let feed_timeout = opts.feed_timeout;

    let server = http_utils::start_server(socket_addr, move |addr, req| {
        let aggregator = aggregator.clone();
        async move {
            match (req.method(), req.uri().path().trim_end_matches('/')) {
                // Check that the server is up and running:
                (&Method::GET, "/health") => Ok(Response::new("OK".into())),
                // Subscribe to feed messages:
                (&Method::GET, "/feed") => {
                    log::info!("Opening /feed connection from {:?}", addr);
                    Ok(http_utils::upgrade_to_websocket(
                        req,
                        move |ws_send, ws_recv| async move {
                            let (feed_id, tx_to_aggregator) = aggregator.subscribe_feed();
                            let (mut tx_to_aggregator, mut ws_send) =
                                handle_feed_websocket_connection(
                                    ws_send,
                                    ws_recv,
                                    tx_to_aggregator,
                                    feed_timeout,
                                    feed_id,
                                )
                                .await;
                            log::info!("Closing /feed connection from {:?}", addr);
                            // Tell the aggregator that this connection has closed, so it can tidy up.
                            let _ = tx_to_aggregator.send(FromFeedWebsocket::Disconnected).await;
                            let _ = ws_send.close().await;
                        },
                    ))
                }
                // Subscribe to shard messages:
                (&Method::GET, "/shard_submit") => {
                    Ok(http_utils::upgrade_to_websocket(
                        req,
                        move |ws_send, ws_recv| async move {
                            log::info!("Opening /shard_submit connection from {:?}", addr);
                            let tx_to_aggregator = aggregator.subscribe_shard();
                            let (mut tx_to_aggregator, mut ws_send) =
                                handle_shard_websocket_connection(
                                    ws_send,
                                    ws_recv,
                                    tx_to_aggregator,
                                )
                                .await;
                            log::info!("Closing /shard_submit connection from {:?}", addr);
                            // Tell the aggregator that this connection has closed, so it can tidy up.
                            let _ = tx_to_aggregator
                                .send(FromShardWebsocket::Disconnected)
                                .await;
                            let _ = ws_send.close().await;
                        },
                    ))
                }
                // Return metrics in a prometheus-friendly text based format:
                (&Method::GET, "/metrics") => Ok(return_prometheus_metrics(aggregator).await),
                // 404 for anything else:
                _ => Ok(Response::builder()
                    .status(404)
                    .body("Not found".into())
                    .unwrap()),
            }
        }
    });

    server.await?;
    Ok(())
}

/// This handles messages coming to/from a shard connection
async fn handle_shard_websocket_connection<S>(
    mut ws_send: http_utils::WsSender,
    mut ws_recv: http_utils::WsReceiver,
    mut tx_to_aggregator: S,
) -> (S, http_utils::WsSender)
where
    S: futures::Sink<FromShardWebsocket, Error = anyhow::Error> + Unpin + Send + 'static,
{
    let (tx_to_shard_conn, rx_from_aggregator) = flume::unbounded();

    // Tell the aggregator about this new connection, and give it a way to send messages to us:
    let init_msg = FromShardWebsocket::Initialize {
        channel: tx_to_shard_conn,
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Error sending message to aggregator: {}", e);
        return (tx_to_aggregator, ws_send);
    }

    // Channels to notify each loop if the other closes:
    let (recv_closer_tx, mut recv_closer_rx) = tokio::sync::oneshot::channel::<()>();
    let (send_closer_tx, mut send_closer_rx) = tokio::sync::oneshot::channel::<()>();

    // Receive messages from a shard:
    let recv_handle = tokio::spawn(async move {
        loop {
            let mut bytes = Vec::new();

            // Receive a message, or bail if closer called. We don't care about cancel safety;
            // if we're halfway through receiving a message, no biggie since we're closing the
            // connection anyway.
            let msg_info = tokio::select! {
                msg_info = ws_recv.receive_data(&mut bytes) => msg_info,
                _ = &mut recv_closer_rx => break
            };

            // Handle the socket closing, or errors receiving the message.
            if let Err(soketto::connection::Error::Closed) = msg_info {
                break;
            }
            if let Err(e) = msg_info {
                log::error!(
                    "Shutting down websocket connection: Failed to receive data: {}",
                    e
                );
                break;
            }

            let msg: internal_messages::FromShardAggregator =
                match bincode::options().deserialize(&bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!(
                            "Failed to deserialize message from shard; booting it: {}",
                            e
                        );
                        break;
                    }
                };

            // Convert and send to the aggregator:
            let aggregator_msg = match msg {
                internal_messages::FromShardAggregator::AddNode {
                    ip,
                    node,
                    local_id,
                    genesis_hash,
                } => FromShardWebsocket::Add {
                    ip,
                    node,
                    genesis_hash,
                    local_id,
                },
                internal_messages::FromShardAggregator::UpdateNode { payload, local_id } => {
                    FromShardWebsocket::Update { local_id, payload }
                }
                internal_messages::FromShardAggregator::RemoveNode { local_id } => {
                    FromShardWebsocket::Remove { local_id }
                }
            };

            if let Err(e) = tx_to_aggregator.send(aggregator_msg).await {
                log::error!("Failed to send message to aggregator; closing shard: {}", e);
                break;
            }
        }

        drop(send_closer_tx); // Kill the send task if this recv task ends
        tx_to_aggregator
    });

    // Send messages to the shard:
    let send_handle = tokio::spawn(async move {
        loop {
            let msg = tokio::select! {
                msg = rx_from_aggregator.recv_async() => msg,
                _ = &mut send_closer_rx => { break }
            };

            let msg = match msg {
                Ok(msg) => msg,
                Err(flume::RecvError::Disconnected) => break,
            };

            let internal_msg = match msg {
                ToShardWebsocket::Mute { local_id, reason } => {
                    internal_messages::FromTelemetryCore::Mute { local_id, reason }
                }
            };

            let bytes = bincode::options()
                .serialize(&internal_msg)
                .expect("message to shard should serialize");

            if let Err(e) = ws_send.send_binary(bytes).await {
                log::error!("Failed to send message to aggregator; closing shard: {}", e)
            }
            if let Err(e) = ws_send.flush().await {
                log::error!(
                    "Failed to flush message to aggregator; closing shard: {}",
                    e
                )
            }
        }

        drop(recv_closer_tx); // Kill the recv task if this send task ends
        ws_send
    });

    // If our send/recv tasks are stopped (if one of them dies, they both will),
    // collect the bits we need to hand back from them:
    let ws_send = send_handle.await.unwrap();
    let tx_to_aggregator = recv_handle.await.unwrap();

    // loop ended; give socket back to parent:
    (tx_to_aggregator, ws_send)
}

/// This handles messages coming from a feed connection
async fn handle_feed_websocket_connection<S>(
    mut ws_send: http_utils::WsSender,
    mut ws_recv: http_utils::WsReceiver,
    mut tx_to_aggregator: S,
    feed_timeout: u64,
    _feed_id: u64, // <- can be useful for debugging purposes.
) -> (S, http_utils::WsSender)
where
    S: futures::Sink<FromFeedWebsocket, Error = anyhow::Error> + Unpin + Send + 'static,
{
    // unbounded channel so that slow feeds don't block aggregator progress:
    let (tx_to_feed_conn, rx_from_aggregator) = flume::unbounded();

    // `Receiver::into_stream()` is currently problematic at the time of writing
    // (see https://github.com/zesterer/flume/issues/88). If this stream is polled lots
    // and isn't ready, it'll leak memory. In this case, since we only select from it or
    // a close channel, we shouldn't poll the thing more than once before it's ready (and
    // when it's ready, it cleans up after itself properly). So, I hope it won't leak!
    let mut rx_from_aggregator_chunks = ReadyChunksAll::new(rx_from_aggregator.into_stream());

    // Tell the aggregator about this new connection, and give it a way to send messages to us:
    let init_msg = FromFeedWebsocket::Initialize {
        channel: tx_to_feed_conn,
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Error sending message to aggregator: {}", e);
        return (tx_to_aggregator, ws_send);
    }

    // Channels to notify each loop if the other closes:
    let (recv_closer_tx, mut recv_closer_rx) = tokio::sync::oneshot::channel::<()>();
    let (send_closer_tx, mut send_closer_rx) = tokio::sync::oneshot::channel::<()>();

    // Receive messages from the feed:
    let recv_handle = tokio::spawn(async move {
        loop {
            let mut bytes = Vec::new();
            // Receive a message, or bail if closer called. We don't care about cancel safety;
            // if we're halfway through receiving a message, no biggie since we're closing the
            // connection anyway.
            let msg_info = tokio::select! {
                msg_info = ws_recv.receive_data(&mut bytes) => msg_info,
                _ = &mut recv_closer_rx => { break }
            };

            // Handle the socket closing, or errors receiving the message.
            if let Err(soketto::connection::Error::Closed) = msg_info {
                break;
            }
            if let Err(e) = msg_info {
                log::error!(
                    "Shutting down websocket connection: Failed to receive data: {}",
                    e
                );
                break;
            }

            // We ignore all but valid UTF8 text messages from the frontend:
            let text = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Parse the message into a command we understand and send it to the aggregator:
            let cmd = match FromFeedWebsocket::from_str(&text) {
                Ok(cmd) => cmd,
                Err(e) => {
                    log::warn!(
                        "Ignoring invalid command '{}' from the frontend: {}",
                        text,
                        e
                    );
                    continue;
                }
            };
            if let Err(e) = tx_to_aggregator.send(cmd).await {
                log::error!("Failed to send message to aggregator; closing feed: {}", e);
                break;
            }
        }

        drop(send_closer_tx); // Kill the send task if this recv task ends
        tx_to_aggregator
    });

    // Send messages to the feed:
    let send_handle = tokio::spawn(async move {
        'outer: loop {
            let debounce = tokio::time::sleep_until(Instant::now() + Duration::from_millis(75));

            let msgs = tokio::select! {
                msgs = rx_from_aggregator_chunks.next() => msgs,
                _ = &mut send_closer_rx => { break }
            };

            // End the loop when connection from aggregator ends:
            let msgs = match msgs {
                Some(msgs) => msgs,
                None => break,
            };

            // There is only one message type at the mo; bytes to send
            // to the websocket. collect them all up to dispatch in one shot.
            let all_msg_bytes = msgs.into_iter().map(|msg| match msg {
                ToFeedWebsocket::Bytes(bytes) => bytes,
            });

            // If the feed is too slow to receive the current batch of messages, we'll drop it.
            let message_send_deadline = Instant::now() + Duration::from_secs(feed_timeout);

            for bytes in all_msg_bytes {
                match tokio::time::timeout_at(message_send_deadline, ws_send.send_binary(&bytes))
                    .await
                {
                    Err(_) => {
                        log::debug!("Closing feed websocket that was too slow to keep up (too slow to send messages)");
                        break 'outer;
                    }
                    Ok(Err(soketto::connection::Error::Closed)) => {
                        break 'outer;
                    }
                    Ok(Err(e)) => {
                        log::debug!("Closing feed websocket due to error sending data: {}", e);
                        break 'outer;
                    }
                    Ok(_) => {}
                }
            }

            match tokio::time::timeout_at(message_send_deadline, ws_send.flush()).await {
                Err(_) => {
                    log::debug!("Closing feed websocket that was too slow to keep up (too slow to flush messages)");
                    break;
                }
                Ok(Err(soketto::connection::Error::Closed)) => {
                    break;
                }
                Ok(Err(e)) => {
                    log::debug!("Closing feed websocket due to error flushing data: {}", e);
                    break;
                }
                Ok(_) => {}
            }

            debounce.await;
        }

        drop(recv_closer_tx); // Kill the recv task if this send task ends
        ws_send
    });

    // If our send/recv tasks are stopped (if one of them dies, they both will),
    // collect the bits we need to hand back from them:
    let ws_send = send_handle.await.unwrap();
    let tx_to_aggregator = recv_handle.await.unwrap();

    // loop ended; give socket back to parent:
    (tx_to_aggregator, ws_send)
}

async fn return_prometheus_metrics(aggregator: AggregatorSet) -> Response<hyper::Body> {
    let metrics = aggregator.latest_metrics();

    // Instead of using the rust prometheus library (which is optimised around global variables updated across a codebase),
    // we just split out the text format that prometheus expects ourselves, and use the latest metrics that we've
    // captured so far from the aggregators. See:
    //
    // https://github.com/prometheus/docs/blob/master/content/docs/instrumenting/exposition_formats.md#text-format-details
    //
    // For an example and explanation of this text based format. The minimal output we produce here seems to
    // be handled correctly when pointing a current version of prometheus at it.
    //
    // Note: '{{' and '}}' are just escaped versions of '{' and '}' in Rust fmt strings.
    use std::fmt::Write;
    let mut s = String::new();
    for (idx, m) in metrics.iter().enumerate() {
        let _ = write!(
            &mut s,
            "telemetry_core_connected_feeds{{aggregator=\"{}\"}} {} {}\n",
            idx, m.connected_feeds, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_connected_nodes{{aggregator=\"{}\"}} {} {}\n",
            idx, m.connected_nodes, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_connected_shards{{aggregator=\"{}\"}} {} {}\n",
            idx, m.connected_shards, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_chains_subscribed_to{{aggregator=\"{}\"}} {} {}\n",
            idx, m.chains_subscribed_to, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_subscribed_feeds{{aggregator=\"{}\"}} {} {}\n",
            idx, m.subscribed_feeds, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_total_messages_to_feeds{{aggregator=\"{}\"}} {} {}\n",
            idx, m.total_messages_to_feeds, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_current_messages_to_aggregator{{aggregator=\"{}\"}} {} {}\n\n",
            idx, m.current_messages_to_aggregator, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_total_messages_to_aggregator{{aggregator=\"{}\"}} {} {}\n\n",
            idx, m.total_messages_to_aggregator, m.timestamp_unix_ms
        );
        let _ = write!(
            &mut s,
            "telemetry_core_dropped_messages_to_aggregator{{aggregator=\"{}\"}} {} {}\n\n",
            idx, m.dropped_messages_to_aggregator, m.timestamp_unix_ms
        );
    }

    Response::builder()
        // The version number here tells prometheus which version of the text format we're using:
        .header(http::header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(s.into())
        .unwrap()
}
