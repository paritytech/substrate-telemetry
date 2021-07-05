mod aggregator;
mod feed_message;
mod find_location;
mod state;

use std::net::SocketAddr;
use std::str::FromStr;

use aggregator::{
    Aggregator, FromFeedWebsocket, FromShardWebsocket, ToFeedWebsocket, ToShardWebsocket,
};
use bincode::Options;
use common::internal_messages;
use futures::{channel::mpsc, SinkExt, StreamExt};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use warp::filters::ws;
use warp::Filter;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = "Substrate Telemetry Backend Core";
const ABOUT: &str = "This is the Telemetry Backend Core that receives telemetry messages \
                     from Substrate/Polkadot nodes and provides the data to a subsribed feed";

#[derive(StructOpt, Debug)]
#[structopt(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
    /// This is the socket address that Telemetryis listening to. This is restricted to
    /// localhost (127.0.0.1) by default and should be fine for most use cases. If
    /// you are using Telemetry in a container, you likely want to set this to '0.0.0.0:8000'
    #[structopt(short = "l", long = "listen", default_value = "127.0.0.1:8000")]
    socket: std::net::SocketAddr,
    /// The desired log level; one of 'error', 'warn', 'info', 'debug' or 'trace', where
    /// 'error' only logs errors and 'trace' logs everything.
    #[structopt(required = false, long = "log", default_value = "info")]
    log_level: log::LevelFilter,
    /// Space delimited list of the names of chains that are not allowed to connect to
    /// telemetry. Case sensitive.
    #[structopt(required = false, long = "denylist")]
    denylist: Vec<String>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();

    SimpleLogger::new()
        .with_level(opts.log_level)
        .init()
        .expect("Must be able to start a logger");

    log::info!("Starting Telemetry Core version: {}", VERSION);

    if let Err(e) = start_server(opts).await {
        log::error!("Error starting server: {}", e);
    }
}

/// Declare our routes and start the server.
async fn start_server(opts: Opts) -> anyhow::Result<()> {
    let shard_aggregator = Aggregator::spawn(opts.denylist).await?;
    let feed_aggregator = shard_aggregator.clone();

    // Handle requests to /health by returning OK.
    let health_route = warp::path("health").map(|| "OK");

    // Handle websocket requests from shards.
    let ws_shard_submit_route = warp::path("shard_submit")
        .and(warp::ws())
        .and(warp::filters::addr::remote())
        .map(move |ws: ws::Ws, addr: Option<SocketAddr>| {
            let tx_to_aggregator = shard_aggregator.subscribe_shard();
            log::info!("Opening /shard_submit connection from {:?}", addr);
            ws.on_upgrade(move |websocket| async move {
                let (mut tx_to_aggregator, websocket) =
                    handle_shard_websocket_connection(websocket, tx_to_aggregator).await;
                log::info!("Closing /shard_submit connection from {:?}", addr);
                // Tell the aggregator that this connection has closed, so it can tidy up.
                let _ = tx_to_aggregator
                    .send(FromShardWebsocket::Disconnected)
                    .await;
                let _ = websocket.close().await;
            })
        });

    // Handle websocket requests from frontends.
    let ws_feed_route = warp::path("feed")
        .and(warp::ws())
        .and(warp::filters::addr::remote())
        .map(move |ws: ws::Ws, addr: Option<SocketAddr>| {
            let tx_to_aggregator = feed_aggregator.subscribe_feed();
            log::info!("Opening /feed connection from {:?}", addr);

            // We can decide how many messages can be buffered to be sent, but not specifically how
            // large those messages are cumulatively allowed to be:
            ws.max_send_queue(1_000 ).on_upgrade(move |websocket| async move {
                let (mut tx_to_aggregator, websocket) =
                    handle_feed_websocket_connection(websocket, tx_to_aggregator).await;
                log::info!("Closing /feed connection from {:?}", addr);
                // Tell the aggregator that this connection has closed, so it can tidy up.
                let _ = tx_to_aggregator.send(FromFeedWebsocket::Disconnected).await;
                let _ = websocket.close().await;
            })
        });

    // Merge the routes and start our server:
    let routes = ws_shard_submit_route.or(ws_feed_route).or(health_route);
    warp::serve(routes).run(opts.socket).await;
    Ok(())
}

/// This handles messages coming to/from a shard connection
async fn handle_shard_websocket_connection<S>(
    mut websocket: ws::WebSocket,
    mut tx_to_aggregator: S,
) -> (S, ws::WebSocket)
where
    S: futures::Sink<FromShardWebsocket, Error = anyhow::Error> + Unpin,
{
    let (tx_to_shard_conn, mut rx_from_aggregator) = mpsc::channel(10);

    // Tell the aggregator about this new connection, and give it a way to send messages to us:
    let init_msg = FromShardWebsocket::Initialize {
        channel: tx_to_shard_conn,
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Error sending message to aggregator: {}", e);
        return (tx_to_aggregator, websocket);
    }

    // Loop, handling new messages from the shard or from the aggregator:
    loop {
        tokio::select! {
            // AGGREGATOR -> SHARD
            msg = rx_from_aggregator.next() => {
                // End the loop when connection from aggregator ends:
                let msg = match msg {
                    Some(msg) => msg,
                    None => break
                };

                let internal_msg = match msg {
                    ToShardWebsocket::Mute { local_id, reason } => {
                        internal_messages::FromTelemetryCore::Mute { local_id, reason }
                    }
                };

                let bytes = bincode::options()
                    .serialize(&internal_msg)
                    .expect("message to shard should serialize");

                if let Err(e) = websocket.send(ws::Message::binary(bytes)).await {
                    log::error!("Error sending message to shard; booting it: {}", e);
                    break
                }
            }
            // SHARD -> AGGREGATOR
            msg = websocket.next() => {
                // End the loop when connection from shard ends:
                let msg = match msg {
                    Some(msg) => msg,
                    None => break
                };

                let msg = match msg {
                    Err(e) => {
                        log::error!("Error receiving message from shard; booting it: {}", e);
                        break;
                    },
                    Ok(msg) => msg
                };

                // Close message? Break and allow connection to be dropped.
                if msg.is_close() {
                    break;
                }

                // If the message isn't something we want to handle, just ignore it.
                // This includes system messages like "pings" and such, so don't log anything.
                if !msg.is_binary() && !msg.is_text() {
                    continue;
                }

                let bytes = msg.as_bytes();
                let msg: internal_messages::FromShardAggregator = match bincode::options().deserialize(bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!("Failed to deserialize message from shard; booting it: {}", e);
                        break;
                    }
                };

                // Convert and send to the aggregator:
                let aggregator_msg = match msg {
                    internal_messages::FromShardAggregator::AddNode { ip, node, local_id, genesis_hash } => {
                        FromShardWebsocket::Add { ip, node, genesis_hash, local_id }
                    },
                    internal_messages::FromShardAggregator::UpdateNode { payload, local_id } => {
                        FromShardWebsocket::Update { local_id, payload }
                    },
                    internal_messages::FromShardAggregator::RemoveNode { local_id } => {
                        FromShardWebsocket::Remove { local_id }
                    },
                };
                if let Err(e) = tx_to_aggregator.send(aggregator_msg).await {
                    log::error!("Failed to send message to aggregator; closing shard: {}", e);
                    break;
                }
            }
        }
    }

    // loop ended; give socket back to parent:
    (tx_to_aggregator, websocket)
}

/// This handles messages coming from a feed connection
async fn handle_feed_websocket_connection<S>(
    websocket: ws::WebSocket,
    mut tx_to_aggregator: S,
) -> (S, ws::WebSocket)
where
    S: futures::Sink<FromFeedWebsocket, Error = anyhow::Error> + Unpin,
{
    // unbounded channel so that slow feeds don't block aggregator progress:
    let (tx_to_feed_conn, mut rx_from_aggregator) = mpsc::unbounded();

    // Tell the aggregator about this new connection, and give it a way to send messages to us:
    let init_msg = FromFeedWebsocket::Initialize {
        channel: tx_to_feed_conn,
    };
    if let Err(e) = tx_to_aggregator.send(init_msg).await {
        log::error!("Error sending message to aggregator: {}", e);
        return (tx_to_aggregator, websocket);
    }

    // Split the socket so that we can poll for flushing and receiving messages simultaneously.
    let (mut ws_sink, mut ws_stream) = websocket.split();

    let mut needs_flush = false;

    // Loop, handling new messages from the shard or from the aggregator:
    loop {
        tokio::select! {

            // AGGREGATOR -> FRONTEND (flush messages while waiting to recv/buffer new ones)
            msg = ws_sink.flush(), if needs_flush => {
                needs_flush = false;
                if let Err(e) = msg {
                    log::error!("Closing feed websocket due to error flushing data: {}", e);
                    break;
                }
            }

            // AGGREGATOR -> FRONTEND (buffer messages to the UI)
            msg = rx_from_aggregator.next() => {
                // End the loop when connection from aggregator ends:
                let msg = match msg {
                    Some(msg) => msg,
                    None => break
                };

                // Send messages to the client (currently the only message is
                // pre-serialized bytes that we send as binary):
                let bytes = match msg {
                    ToFeedWebsocket::Bytes(bytes) => bytes
                };

                log::debug!("Message to feed: {}", std::str::from_utf8(&bytes).unwrap_or("INVALID UTF8"));

                // `start_send` internally calls tungstenite's `write_message`, which returns an error if the
                // message buffer is full. If we awaited on a flush here, then a slow client would cause backpressure
                // which would fill the unbounded channel to the aggregator.
                //
                // Normally we should call `poll_ready` first to confirm that we can send a thing to the Sink, but
                // in this case it just calls tungstenite's `write_pending` to try flush the buffer, and we deliberately
                // _don't_ want to try and drain the buffer here. Instead, we attempt to flush concurrently in this select
                // loop, and if the buffer fills we get an error back here and just close the socket
                if let Err(e) = ws_sink.start_send_unpin(ws::Message::binary(bytes)) {
                    log::warn!("Closing feed websocket due to error buffering message: {}", e);
                    break;
                }
                needs_flush = true;
            }

            // FRONTEND -> AGGREGATOR (relay messages to the aggregator)
            msg = ws_stream.next() => {
                // End the loop when connection from feed ends:
                let msg = match msg {
                    Some(msg) => msg,
                    None => break
                };

                // If we see any errors, log them and end our loop:
                let msg = match msg {
                    Err(e) => {
                        log::error!("Error in node websocket connection: {}", e);
                        break;
                    },
                    Ok(msg) => msg
                };

                // Close message? Break and allow connection to be dropped.
                if msg.is_close() {
                    break;
                }

                // We ignore all but text messages from the frontend:
                let text = match msg.to_str() {
                    Ok(s) => s,
                    Err(_) => continue
                };

                // Parse the message into a command we understand and send it to the aggregator:
                let cmd = match FromFeedWebsocket::from_str(text) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        log::warn!("Ignoring invalid command '{}' from the frontend: {}", text, e);
                        continue
                    }
                };
                if let Err(e) = tx_to_aggregator.send(cmd).await {
                    log::error!("Failed to send message to aggregator; closing feed: {}", e);
                    break;
                }
            }
        }
    }

    let websocket = ws_sink.reunite(ws_stream).expect("Reunite should always succeed");
    // loop ended; give socket back to parent:
    (tx_to_aggregator, websocket)
}
