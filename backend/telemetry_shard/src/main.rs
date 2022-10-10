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

#[warn(missing_docs)]
mod aggregator;
mod blocked_addrs;
mod connection;
mod json_message;
mod real_ip;

use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use aggregator::{Aggregator, FromWebsocket};
use blocked_addrs::BlockedAddrs;
use common::byte_size::ByteSize;
use common::http_utils;
use common::node_message;
use common::node_message::NodeMessageId;
use common::rolling_total::RollingTotalBuilder;
use futures::{SinkExt, StreamExt};
use http::Uri;
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
const NAME: &str = "Substrate Telemetry Backend Shard";
const ABOUT: &str = "This is the Telemetry Backend Shard that forwards the \
                     data sent by Substrate/Polkadot nodes to the Backend Core";

#[derive(StructOpt, Debug)]
#[structopt(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
    /// This is the socket address that this shard is listening to. This is restricted to
    /// localhost (127.0.0.1) by default and should be fine for most use cases. If
    /// you are using Telemetry in a container, you likely want to set this to '0.0.0.0:8000'
    #[structopt(short = "l", long = "listen", default_value = "127.0.0.1:8001")]
    socket: std::net::SocketAddr,
    /// The desired log level; one of 'error', 'warn', 'info', 'debug' or 'trace', where
    /// 'error' only logs errors and 'trace' logs everything.
    #[structopt(long = "log", default_value = "info")]
    log_level: log::LevelFilter,
    /// Url to the Backend Core endpoint accepting shard connections
    #[structopt(
        short = "c",
        long = "core",
        default_value = "ws://127.0.0.1:8000/shard_submit/"
    )]
    core_url: Uri,
    /// How many different nodes is a given connection to the /submit endpoint allowed to
    /// tell us about before we ignore the rest?
    ///
    /// This is important because without a limit, a single connection could exhaust
    /// RAM by suggesting that it accounts for billions of nodes.
    #[structopt(long, default_value = "20")]
    max_nodes_per_connection: usize,
    /// What is the maximum number of bytes per second, on average, that a connection from a
    /// node is allowed to send to a shard before it gets booted. This is averaged over a
    /// rolling window of 10 seconds, and so spikes beyond this limit are allowed as long as
    /// the average traffic in the last 10 seconds falls below this value.
    ///
    /// As a reference point, syncing a new Polkadot node leads to a maximum of about 25k of
    /// traffic on average (at least initially).
    #[structopt(long, default_value = "256k")]
    max_node_data_per_second: ByteSize,
    /// How many seconds is a "/feed" connection that violates the '--max-node-data-per-second'
    /// value prevented from reconnecting to this shard for, in seconds.
    #[structopt(long, default_value = "600")]
    node_block_seconds: u64,
    /// Number of worker threads to spawn. If "0" is given, use the number of CPUs available
    /// on the machine. If no value is given, use an internal default that we have deemed sane.
    #[structopt(long)]
    worker_threads: Option<usize>,
    /// Roughly how long to wait in seconds for new telemetry data to arrive from a node. If
    /// telemetry for a node does not arrive in this time frame, we remove the corresponding node
    /// state, and if no messages are received on the connection at all in this time, it will be
    /// dropped.
    #[structopt(long, default_value = "60")]
    stale_node_timeout: u64,
}

fn main() {
    let opts = Opts::from_args();

    SimpleLogger::new()
        .with_level(opts.log_level)
        .init()
        .expect("Must be able to start a logger");

    log::info!("Starting Telemetry Shard version: {}", VERSION);

    let worker_threads = match opts.worker_threads {
        Some(0) => num_cpus::get(),
        Some(n) => n,
        // By default, use a max of 4 worker threads, as we don't
        // expect to need a lot of parallelism in shards.
        None => usize::min(num_cpus::get(), 4),
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(worker_threads)
        .thread_name("telemetry_shard_worker")
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
    let block_list = BlockedAddrs::new(Duration::from_secs(opts.node_block_seconds));
    let aggregator = Aggregator::spawn(opts.core_url).await?;
    let socket_addr = opts.socket;
    let max_nodes_per_connection = opts.max_nodes_per_connection;
    let bytes_per_second = opts.max_node_data_per_second;
    let stale_node_timeout = Duration::from_secs(opts.stale_node_timeout);

    let server = http_utils::start_server(socket_addr, move |addr, req| {
        let aggregator = aggregator.clone();
        let block_list = block_list.clone();
        async move {
            match (req.method(), req.uri().path().trim_end_matches('/')) {
                // Check that the server is up and running:
                (&Method::GET, "/health") => Ok(Response::new("OK".into())),
                // Nodes send messages here:
                (&Method::GET, "/submit") => {
                    let (real_addr, real_addr_source) = real_ip::real_ip(addr, req.headers());

                    if let Some(reason) = block_list.blocked_reason(&real_addr) {
                        return Ok(Response::builder().status(403).body(reason.into()).unwrap());
                    }

                    Ok(http_utils::upgrade_to_websocket(
                        req,
                        move |ws_send, ws_recv| async move {
                            log::info!(
                                "Opening /submit connection from {:?} (address source: {})",
                                real_addr,
                                real_addr_source
                            );
                            let tx_to_aggregator = aggregator.subscribe_node();
                            let (mut tx_to_aggregator, mut ws_send) =
                                handle_node_websocket_connection(
                                    real_addr,
                                    ws_send,
                                    ws_recv,
                                    tx_to_aggregator,
                                    max_nodes_per_connection,
                                    bytes_per_second,
                                    block_list,
                                    stale_node_timeout,
                                )
                                .await;
                            log::info!(
                                "Closing /submit connection from {:?} (address source: {})",
                                real_addr,
                                real_addr_source
                            );
                            // Tell the aggregator that this connection has closed, so it can tidy up.
                            let _ = tx_to_aggregator.send(FromWebsocket::Disconnected).await;
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

/// This takes care of handling messages from an established socket connection.
async fn handle_node_websocket_connection<S>(
    real_addr: IpAddr,
    ws_send: http_utils::WsSender,
    mut ws_recv: http_utils::WsReceiver,
    mut tx_to_aggregator: S,
    max_nodes_per_connection: usize,
    bytes_per_second: ByteSize,
    block_list: BlockedAddrs,
    stale_node_timeout: Duration,
) -> (S, http_utils::WsSender)
where
    S: futures::Sink<FromWebsocket, Error = anyhow::Error> + Unpin + Send + 'static,
{
    // Keep track of the message Ids that have been "granted access". We allow a maximum of
    // `max_nodes_per_connection` before ignoring others.
    let mut allowed_message_ids = HashMap::<NodeMessageId, Instant>::new();

    // Limit the number of bytes based on a rolling total and the incoming bytes per second
    // that has been configured via the CLI opts.
    let bytes_per_second = bytes_per_second.num_bytes();
    let mut rolling_total_bytes = RollingTotalBuilder::new()
        .granularity(Duration::from_secs(1))
        .window_size_multiple(10)
        .start();

    // This could be a oneshot channel, but it's useful to be able to clone
    // messages, and we can't clone oneshot channel senders.
    let (close_connection_tx, close_connection_rx) = flume::bounded(1);

    // Tell the aggregator about this new connection, and give it a way to close this connection:
    let init_msg = FromWebsocket::Initialize {
        close_connection: close_connection_tx.clone(),
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Shutting down websocket connection from {real_addr:?}: Error sending message to aggregator: {e}");
        return (tx_to_aggregator, ws_send);
    }

    // Receiving data isn't cancel safe, so let it happen in a separate task.
    // If this loop ends, the outer will receive a `None` message and end too.
    // If the outer loop ends, it fires a msg on `close_connection_rx` to ensure this ends too.
    let (ws_tx_atomic, mut ws_rx_atomic) = futures::channel::mpsc::unbounded();
    tokio::task::spawn(async move {
        loop {
            let mut bytes = Vec::new();
            tokio::select! {
                // The close channel has fired, so end the loop. `ws_recv.receive_data` is
                // *not* cancel safe, but since we're closing the connection we don't care.
                _ = close_connection_rx.recv_async() => {
                    log::info!("connection to {real_addr:?} being closed");
                    break
                },
                // Receive data and relay it on to our main select loop below.
                msg_info = ws_recv.receive_data(&mut bytes) => {
                    if let Err(soketto::connection::Error::Closed) = msg_info {
                        break;
                    }
                    if let Err(e) = msg_info {
                        log::error!("Shutting down websocket connection from {real_addr:?}: Failed to receive data: {e}");
                        break;
                    }
                    if ws_tx_atomic.unbounded_send(bytes).is_err() {
                        // The other end closed; end this loop.
                        break;
                    }
                }
            }
        }
    });

    // A periodic interval to check for stale nodes.
    let mut stale_interval = tokio::time::interval(stale_node_timeout / 2);

    // Our main select loop atomically receives and handles telemetry messages from the node,
    // and periodically checks for stale connections to keep our node state tidy.
    loop {
        tokio::select! {
            // We periodically check for stale message IDs and remove nodes associated with
            // them, to prevent a buildup. We boot the whole connection if no interpretable
            // messages have been sent at all in the time period.
            _ = stale_interval.tick() => {
                let stale_ids: Vec<NodeMessageId> = allowed_message_ids.iter()
                    .filter(|(_, last_seen)| last_seen.elapsed() > stale_node_timeout)
                    .map(|(&id, _)| id)
                    .collect();

                for &message_id in &stale_ids {
                    log::info!("Removing stale node with message ID {message_id} from {real_addr:?}");
                    allowed_message_ids.remove(&message_id);
                    let _ = tx_to_aggregator.send(FromWebsocket::Remove { message_id } ).await;
                }

                if !stale_ids.is_empty() && allowed_message_ids.is_empty() {
                    // End the entire connection if no recent messages came in for any ID.
                    log::info!("Closing stale connection from {real_addr:?}");
                    break;
                }
            },
            // Handle messages received by the connected node.
            msg = ws_rx_atomic.next() => {
                // No more messages? break.
                let bytes = match msg {
                    Some(bytes) => bytes,
                    None => { break; }
                };

                // Keep track of total bytes and bail if average over last 10 secs exceeds preference.
                rolling_total_bytes.push(bytes.len());
                let this_bytes_per_second = rolling_total_bytes.total() / 10;
                if this_bytes_per_second > bytes_per_second {
                    block_list.block_addr(real_addr, "Too much traffic");
                    log::error!("Shutting down websocket connection: Too much traffic ({this_bytes_per_second}bps averaged over last 10s)");
                    break;
                }

                // Deserialize from JSON, warning in debug mode if deserialization fails:
                let node_message: json_message::NodeMessage = match serde_json::from_slice(&bytes) {
                    Ok(node_message) => node_message,
                    #[cfg(debug)]
                    Err(e) => {
                        let bytes: &[u8] = bytes.get(..512).unwrap_or_else(|| &bytes);
                        let msg_start = std::str::from_utf8(bytes).unwrap_or_else(|_| "INVALID UTF8");
                        log::warn!("Failed to parse node message ({msg_start}): {e}");
                        continue;
                    },
                    #[cfg(not(debug))]
                    Err(_) => {
                        continue;
                    }
                };

                // Pull relevant details from the message:
                let node_message: node_message::NodeMessage = node_message.into();
                let message_id = node_message.id();
                let payload = node_message.into_payload();

                // Until the aggregator receives an `Add` message, which we can create once
                // we see one of these SystemConnected ones, it will ignore messages with
                // the corresponding message_id.
                if let node_message::Payload::SystemConnected(info) = payload {
                    // Too many nodes seen on this connection? Ignore this one.
                    if allowed_message_ids.len() >= max_nodes_per_connection {
                        log::info!("Ignoring new node with ID {message_id} from {real_addr:?} (we've hit the max of {max_nodes_per_connection} nodes per connection)");
                        continue;
                    }

                    // Note of the message ID, allowing telemetry for it.
                    allowed_message_ids.insert(message_id, Instant::now());

                    // Tell the aggregator loop about the new node.
                    log::info!("Adding node with message ID {message_id} from {real_addr:?}");
                    let _ = tx_to_aggregator.send(FromWebsocket::Add {
                        message_id,
                        ip: real_addr,
                        node: info.node,
                        genesis_hash: info.genesis_hash,
                    }).await;
                }
                // Anything that's not an "Add" is an Update. The aggregator will ignore
                // updates against a message_id that hasn't first been Added, above.
                else {
                    if let Some(last_seen) = allowed_message_ids.get_mut(&message_id) {
                        *last_seen = Instant::now();
                        if let Err(e) = tx_to_aggregator.send(FromWebsocket::Update { message_id, payload } ).await {
                            log::error!("Failed to send node message to aggregator: {e}");
                            continue;
                        }
                    } else {
                        log::info!("Ignoring message with ID {message_id} from {real_addr:?} (we've hit the max of {max_nodes_per_connection} nodes per connection)");
                        continue;
                    }
                }
            }
        }
    }

    // Make sure to kill off the receive-messages task if the main select loop ends:
    let _ = close_connection_tx.send(());

    // Return what we need to close the connection gracefully:
    (tx_to_aggregator, ws_send)
}
