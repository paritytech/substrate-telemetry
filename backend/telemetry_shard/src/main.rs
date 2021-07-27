#[warn(missing_docs)]

mod aggregator;
mod connection;
mod json_message;
mod real_ip;

use std::{collections::HashSet, net::IpAddr};

use aggregator::{Aggregator, FromWebsocket};
use common::node_message;
use futures::{channel::mpsc, SinkExt, StreamExt};
use http::Uri;
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use hyper::{ Response, Method };
use common::http_utils;

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
    max_nodes_per_connection: usize
}

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();

    SimpleLogger::new()
        .with_level(opts.log_level)
        .init()
        .expect("Must be able to start a logger");

    log::info!("Starting Telemetry Shard version: {}", VERSION);

    if let Err(e) = start_server(opts).await {
        log::error!("Error starting server: {}", e);
    }
}

/// Declare our routes and start the server.
async fn start_server(opts: Opts) -> anyhow::Result<()> {
    let aggregator = Aggregator::spawn(opts.core_url).await?;
    let socket_addr = opts.socket;
    let max_nodes_per_connection = opts.max_nodes_per_connection;

    let server = http_utils::start_server(socket_addr, move |addr, req| {
        let aggregator = aggregator.clone();
        async move {
            match (req.method(), req.uri().path().trim_end_matches('/')) {
                // Check that the server is up and running:
                (&Method::GET, "/health") => {
                    Ok(Response::new("OK".into()))
                },
                // Nodes send messages here:
                (&Method::GET, "/submit") => {
                    let real_addr = real_ip::real_ip(addr, req.headers());
                    Ok(http_utils::upgrade_to_websocket(req, move |ws_send, ws_recv| async move {
                        let tx_to_aggregator = aggregator.subscribe_node();
                        let (mut tx_to_aggregator, mut ws_send)
                            = handle_node_websocket_connection(real_addr, ws_send, ws_recv, tx_to_aggregator, max_nodes_per_connection).await;
                        log::info!("Closing /submit connection from {:?}", addr);
                        // Tell the aggregator that this connection has closed, so it can tidy up.
                        let _ = tx_to_aggregator.send(FromWebsocket::Disconnected).await;
                        let _ = ws_send.close().await;
                    }))
                },
                // 404 for anything else:
                _ => {
                    Ok(Response::builder()
                        .status(404)
                        .body("Not found".into())
                        .unwrap())
                }
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
    max_nodes_per_connection: usize
) -> (S, http_utils::WsSender)
where
    S: futures::Sink<FromWebsocket, Error = anyhow::Error> + Unpin + Send + 'static,
{
    // Track all of the message IDs that we've seen so far. If we exceed the
    // max_nodes_per_connection limit we ignore subsequent message IDs.
    let mut message_ids_seen = HashSet::new();

    // This could be a oneshot channel, but it's useful to be able to clone
    // messages, and we can't clone oneshot channel senders.
    let (close_connection_tx, mut close_connection_rx) = mpsc::channel(0);

    // Tell the aggregator about this new connection, and give it a way to close this connection:
    let init_msg = FromWebsocket::Initialize {
        close_connection: close_connection_tx,
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Error sending message to aggregator: {}", e);
        return (tx_to_aggregator, ws_send);
    }

    // Now we've "initialized", wait for messages from the node. Messages will
    // either be `SystemConnected` type messages that inform us that a new set
    // of messages with some message ID will be sent (a node could have more
    // than one of these), or updates linked to a specific message_id.
    loop {
        let mut bytes = Vec::new();
        tokio::select! {
            // The close channel has fired, so end the loop. `ws_recv.receive_data` is
            // *not* cancel safe, but since we're closing the connection we don't care.
            _ = close_connection_rx.next() => {
                log::info!("connection to {:?} being closed by aggregator", real_addr);
                break
            },
            // A message was received; handle it:
            msg_info = ws_recv.receive_data(&mut bytes) => {
                // Handle the socket closing, or errors receiving the message.
                if let Err(soketto::connection::Error::Closed) = msg_info {
                    break;
                }
                if let Err(e) = msg_info {
                    log::error!("Shutting down websocket connection: Failed to receive data: {}", e);
                    break;
                }

                // Deserialize from JSON, warning in debug mode if deserialization fails:
                let node_message: json_message::NodeMessage = match serde_json::from_slice(&bytes) {
                    Ok(node_message) => node_message,
                    #[cfg(debug)]
                    Err(e) => {
                        let bytes: &[u8] = bytes.get(..512).unwrap_or_else(|| &bytes);
                        let msg_start = std::str::from_utf8(bytes).unwrap_or_else(|_| "INVALID UTF8");
                        log::warn!("Failed to parse node message ({}): {}", msg_start, e);
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

                // Ignore messages from IDs that exceed our limit:
                if message_ids_seen.contains(&message_id) {
                    // continue on; we're happy
                } else if message_ids_seen.len() >= max_nodes_per_connection {
                    // ignore this message; it's not a "seen" ID and we've hit our limit.
                    continue;
                } else {
                    // not seen ID, not hit limit; make note of new ID
                    message_ids_seen.insert(message_id);
                }

                // Until the aggregator receives an `Add` message, which we can create once
                // we see one of these SystemConnected ones, it will ignore messages with
                // the corresponding message_id.
                if let node_message::Payload::SystemConnected(info) = payload {
                    let _ = tx_to_aggregator.send(FromWebsocket::Add {
                        message_id,
                        ip: real_addr,
                        node: info.node,
                        genesis_hash: info.genesis_hash,
                    }).await;
                }
                // Anything that's not an "Add" is an Update. The aggregator will ignore
                // updates against a message_id that hasn't first been Added, above.
                else if let Err(e) = tx_to_aggregator.send(FromWebsocket::Update { message_id, payload } ).await {
                    log::error!("Failed to send node message to aggregator: {}", e);
                    continue;
                }
            }
        }
    }

    // Return what we need to close the connection gracefully:
    (tx_to_aggregator, ws_send)
}
