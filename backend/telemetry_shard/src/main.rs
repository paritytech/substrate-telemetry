#[warn(missing_docs)]

mod aggregator;
mod connection;
mod json_message;
mod real_ip;

use std::net::IpAddr;

use aggregator::{Aggregator, FromWebsocket};
use common::node_message;
use futures::{channel::mpsc, SinkExt, StreamExt};
use http::Uri;
use real_ip::real_ip;
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use warp::filters::ws;
use warp::Filter;

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
    #[structopt(required = false, long = "log", default_value = "info")]
    log_level: log::LevelFilter,
    /// Url to the Backend Core endpoint accepting shard connections
    #[structopt(
        short = "c",
        long = "core",
        default_value = "ws://127.0.0.1:8000/shard_submit/"
    )]
    core_url: Uri,
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

    // Handle requests to /health by returning OK.
    let health_route = warp::path("health").map(|| "OK");

    // Handle websocket requests to /submit.
    let ws_route = warp::path("submit").and(warp::ws()).and(real_ip()).map(
        move |ws: ws::Ws, addr: Option<IpAddr>| {
            // Send messages from the websocket connection to this sink
            // to have them pass to the aggregator.
            let tx_to_aggregator = aggregator.subscribe_node();
            log::info!("Opening /submit connection from {:?}", addr);
            ws.on_upgrade(move |websocket| async move {
                let (mut tx_to_aggregator, websocket) =
                    handle_node_websocket_connection(websocket, tx_to_aggregator, addr).await;
                log::info!("Closing /submit connection from {:?}", addr);
                // Tell the aggregator that this connection has closed, so it can tidy up.
                let _ = tx_to_aggregator.send(FromWebsocket::Disconnected).await;
                // Note: IF we want to close with a status code and reason, we need to construct
                // a ws::Message using `ws::Message::close_with`, rather than using this method:
                let _ = websocket.close().await;
            })
        },
    );

    // Merge the routes and start our server:
    let routes = ws_route.or(health_route);
    warp::serve(routes).run(opts.socket).await;
    Ok(())
}

/// This takes care of handling messages from an established socket connection.
async fn handle_node_websocket_connection<S>(
    mut websocket: ws::WebSocket,
    mut tx_to_aggregator: S,
    addr: Option<IpAddr>,
) -> (S, ws::WebSocket)
where
    S: futures::Sink<FromWebsocket, Error = anyhow::Error> + Unpin,
{
    // This could be a oneshot channel, but it's useful to be able to clone
    // messages, and we can't clone oneshot channel senders.
    let (close_connection_tx, mut close_connection_rx) = mpsc::channel(0);

    // Tell the aggregator about this new connection, and give it a way to close this connection:
    let init_msg = FromWebsocket::Initialize {
        close_connection: close_connection_tx,
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Error sending message to aggregator: {}", e);
        return (tx_to_aggregator, websocket);
    }

    // Now we've "initialized", wait for messages from the node. Messages will
    // either be `SystemConnected` type messages that inform us that a new set
    // of messages with some message ID will be sent (a node could have more
    // than one of these), or updates linked to a specific message_id.
    loop {
        tokio::select! {
            // The close channel has fired, so end the loop:
            _ = close_connection_rx.next() => {
                log::info!("connection to {:?} being closed by aggregator", addr);
                break
            },
            // A message was received; handle it:
            msg = websocket.next() => {
                let msg = match msg {
                    Some(msg) => msg,
                    None => { log::warn!("Websocket connection from {:?} closed", addr); break }
                };

                // If we see any errors, log them and end our loop:
                let msg = match msg {
                    Err(e) => { log::error!("Error in node websocket connection: {}", e); break },
                    Ok(msg) => msg,
                };

                // Close message? Break to close connection.
                if msg.is_close() {
                    break;
                }

                // If the message isn't something we want to handle, just ignore it.
                // This includes system messages like "pings" and such, so don't log anything.
                if !msg.is_binary() && !msg.is_text() {
                    continue;
                }
                // Deserialize from JSON, warning in debug mode if deserialization fails:
                let bytes = msg.as_bytes();
                let node_message: json_message::NodeMessage = match serde_json::from_slice(bytes) {
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

                // Until the aggregator receives an `Add` message, which we can create once
                // we see one of these SystemConnected ones, it will ignore messages with
                // the corresponding message_id.
                if let node_message::Payload::SystemConnected(info) = payload {
                    let _ = tx_to_aggregator.send(FromWebsocket::Add {
                        message_id,
                        ip: addr,
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
    (tx_to_aggregator, websocket)
}
