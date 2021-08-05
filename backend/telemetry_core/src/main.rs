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
mod find_location;
mod state;
use std::str::FromStr;
use tokio::time::{Duration, Instant};

use aggregator::{
    Aggregator, FromFeedWebsocket, FromShardWebsocket, ToFeedWebsocket, ToShardWebsocket,
};
use bincode::Options;
use common::http_utils;
use common::internal_messages;
use common::ready_chunks_all::ReadyChunksAll;
use futures::{channel::mpsc, SinkExt, StreamExt};
use hyper::{Method, Response};
use simple_logger::SimpleLogger;
use structopt::StructOpt;

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
    /// Number of worker threads to spawn. Defaults to the number of CPUs on the machine.
    /// If "0" is given, use the number of CPUs available on the machine.
    #[structopt(long)]
    worker_threads: Option<usize>,
}

fn main() {
    let opts = Opts::from_args();

    SimpleLogger::new()
        .with_level(opts.log_level)
        .init()
        .expect("Must be able to start a logger");

    log::info!("Starting Telemetry Core version: {}", VERSION);

    let worker_threads = opts.worker_threads
        .and_then(|n| if n == 0 { None } else { Some(n) })
        .unwrap_or_else(|| num_cpus::get());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(worker_threads)
        .build()
        .unwrap()
        .block_on(async {
            if let Err(e) = start_server(opts).await {
                log::error!("Error starting server: {}", e);
            }
        });
}

/// Declare our routes and start the server.
async fn start_server(opts: Opts) -> anyhow::Result<()> {
    let aggregator = Aggregator::spawn(opts.denylist).await?;
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
    let (tx_to_shard_conn, mut rx_from_aggregator) = mpsc::unbounded();

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
                msg = rx_from_aggregator.next() => msg,
                _ = &mut send_closer_rx => { break }
            };

            let msg = match msg {
                Some(msg) => msg,
                None => break,
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
    feed_id: u64
) -> (S, http_utils::WsSender)
where
    S: futures::Sink<FromFeedWebsocket, Error = anyhow::Error> + Unpin + Send + 'static,
{
    // unbounded channel so that slow feeds don't block aggregator progress:
    let (tx_to_feed_conn, rx_from_aggregator) = mpsc::unbounded();
    let mut rx_from_aggregator_chunks = ReadyChunksAll::new(rx_from_aggregator);

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
let mut i: u64 = 0;
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

if feed_id == 1 {
    i += 1;
    println!("FEED #{}, msgs: {}", i, msgs.len());
}
            // End the loop when there are more than 10k messages queued up.
            // This number is just picked as a fairly high limit that should account
            // for many thousands of nodes on a chain. The higher this number is, the
            // larger our channel storage and memory usage is liable to grow before the feed
            // is dropped.
            if msgs.len() > 100_000 {
                log::warn!("Closing feed websocket that was too slow to keep up (too many messages buffered)");
                break 'outer;
            }

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
                        log::warn!("Closing feed websocket that was too slow to keep up (too slow to send messages)");
                        break 'outer;
                    }
                    Ok(Err(e)) => {
                        log::warn!("Closing feed websocket due to error sending data: {}", e);
                        break 'outer;
                    }
                    Ok(_) => {}
                }
            }
            match tokio::time::timeout_at(message_send_deadline, ws_send.flush()).await {
                Err(_) => {
                    log::warn!("Closing feed websocket that was too slow to keep up (too slow to flush messages)");
                    break;
                }
                Ok(Err(e)) => {
                    log::warn!("Closing feed websocket due to error flushing data: {}", e);
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
