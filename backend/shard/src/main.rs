mod aggregator;
mod connection;

use std::net::SocketAddr;

use structopt::StructOpt;
use http::Uri;
use simple_logger::SimpleLogger;
use futures::{StreamExt, SinkExt, channel::mpsc};
use warp::Filter;
use warp::filters::ws;
use common::{json, node, log_level::LogLevel};
use aggregator::{ Aggregator, FromWebsocket };

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
    #[structopt(
        short = "l",
        long = "listen",
        default_value = "127.0.0.1:8001",
    )]
    socket: std::net::SocketAddr,
    /// The desired log level; one of 'error', 'warn', 'info', 'debug' or 'trace', where
    /// 'error' only logs errors and 'trace' logs everything.
    #[structopt(
        required = false,
        long = "log",
        default_value = "info",
        about = "Log level."
    )]
    log_level: LogLevel,
    /// Url to the Backend Core endpoint accepting shard connections
    #[structopt(
    	short = "c",
    	long = "core",
    	default_value = "ws://127.0.0.1:8000/shard_submit/",
    )]
    core_url: Uri,
}

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();
    let log_level = &opts.log_level;

    SimpleLogger::new()
        .with_level(log_level.into())
        .init()
        .expect("Must be able to start a logger");

    log::info!(
        "Starting Telemetry Shard version: {}",
        VERSION
    );

    if let Err(e) = start_server(opts).await {
        log::error!("Error starting server: {}", e);
    }
}

/// Declare our routes and start the server.
async fn start_server(opts: Opts) -> anyhow::Result<()> {

    let aggregator = Aggregator::spawn(opts.core_url).await?;

    // Handle requests to /health by returning OK.
    let health_route =
        warp::path("health")
        .map(|| "OK");

    // Handle websocket requests to /submit.
    let ws_route =
        warp::path("submit")
        .and(warp::ws())
        .and(warp::filters::addr::remote())
        .map(move |ws: ws::Ws, addr: Option<SocketAddr>| {
            // Send messages from the websocket connection to this sink
            // to have them pass to the aggregator.
            let tx_to_aggregator = aggregator.subscribe_node();
            log::info!("Opening /submit connection from {:?}", addr);
            ws.on_upgrade(move |websocket| async move {
                handle_websocket_connection(websocket, tx_to_aggregator, addr).await;
                log::info!("Closing /submit connection from {:?}", addr);
            })
        });

    // Merge the routes and start our server:
    let routes = ws_route.or(health_route);
    warp::serve(routes).run(opts.socket).await;
    Ok(())
}

/// This takes care of handling messages from an established socket connection.
async fn handle_websocket_connection<S>(websocket: ws::WebSocket, mut tx_to_aggregator: S, addr: Option<SocketAddr>)
    where S: futures::Sink<FromWebsocket, Error = anyhow::Error> + Unpin
{
    let mut websocket = websocket.fuse();

    // This could be a oneshot channel, but it's useful to be able to clone
    // messages, and we can't clone oneshot channel senders.
    let (close_connection_tx, mut close_connection_rx) = mpsc::channel(0);

    // First, we wait until we receive a SystemConnected message.
    // Until this turns up, we ignore other messages. We could buffer
    // a few quite easily if we liked.
    while let Some(msg) = websocket.next().await {
        let node_message = match deserialize_ws_message(msg) {
            Ok(Some(msg)) => msg,
            Ok(None) => continue,
            Err(e) => { log::error!("{}", e); break }
        };

        let message_id = node_message.id();
        let payload = node_message.into_payload();

        if let node::Payload::SystemConnected(info) = payload {
            let _ = tx_to_aggregator.send(FromWebsocket::Add {
                message_id,
                ip: addr.map(|a| a.ip()),
                node: info.node,
                close_connection: close_connection_tx,
            }).await;
            break;
        }
    }

    // Now, the node has been added, so we forward messages along as updates.
    // We keep an eye on the close_connection channel; if that resolves, then
    // end this loop and let the connection close gracefully.
    loop {
        futures::select_biased! {
            // The close channel has fired, so end the loop:
            _ = close_connection_rx.next() => {
                break
            },
            // A message was received; handle it:
            msg = websocket.next() => {
                let msg = match msg {
                    Some(msg) => msg,
                    None => break
                };

                let node_message = match deserialize_ws_message(msg) {
                    Ok(Some(msg)) => msg,
                    Ok(None) => continue,
                    Err(e) => { log::error!("{}", e); break }
                };

                let message_id = node_message.id();
                let payload = node_message.into_payload();

                if let Err(e) = tx_to_aggregator.send(FromWebsocket::Update { message_id, payload } ).await {
                    log::error!("Failed to send node message to aggregator: {}", e);
                    continue;
                }
            }
        }
    }

    // loops ended; attempt to close the connection gracefully:
    let _ = websocket.close().await;
}

/// Deserialize an incoming websocket message, returning an error if something
/// fatal went wrong, [`Some`] message if all went well, and [`None`] if a non-fatal
/// issue was encountered and the message should simply be ignored.
fn deserialize_ws_message(msg: Result<ws::Message, warp::Error>) -> anyhow::Result<Option<node::NodeMessage>> {
    // If we see any errors, log them and end our loop:
    let msg = match msg {
        Err(e) => {
            return Err(anyhow::anyhow!("Error in node websocket connection: {}", e));
        },
        Ok(msg) => msg
    };

    // If the message isn't something we want to handle, just ignore it.
    // This includes system messages like "pings" and such, so don't log anything.
    if !msg.is_binary() && !msg.is_text() {
        return Ok(None);
    }

    // Deserialize from JSON, warning if deserialization fails:
    let bytes = msg.as_bytes();
    let node_message: json::NodeMessage = match serde_json::from_slice(bytes) {
        Ok(node_message) => node_message,
        Err(_e) => {
            // let bytes: &[u8] = bytes.get(..512).unwrap_or_else(|| &bytes);
            // let msg_start = std::str::from_utf8(bytes).unwrap_or_else(|_| "INVALID UTF8");
            // log::warn!("Failed to parse node message ({}): {}", msg_start, e);
            return Ok(None)
        }
    };

    // Pull relevant details from the message:
    let node_message: node::NodeMessage = node_message.into();
    Ok(Some(node_message))
}