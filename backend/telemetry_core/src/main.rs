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
use common::ready_chunks_all::ReadyChunksAll;
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
            ws.max_send_queue(1_000)
                .on_upgrade(move |websocket| async move {
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
    let (tx_to_shard_conn, mut rx_from_aggregator) = mpsc::unbounded();

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
    mut websocket: ws::WebSocket,
    mut tx_to_aggregator: S,
) -> (S, ws::WebSocket)
where
    S: futures::Sink<FromFeedWebsocket, Error = anyhow::Error> + Unpin,
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
        return (tx_to_aggregator, websocket);
    }

    // Loop, handling new messages from the shard or from the aggregator:
    loop {
        // Without any special handling, if messages come in every ~2.5ms to each feed, the select! loop
        // has to wake up 400 times a second to poll things. If we have 1000 feeds, that's 400,000 wakeups
        // per second. Even without any work in the loop, that uses a bunch of CPU. As an example, try
        // replacing the loop with this:
        //
        // ```
        // let s = tokio::time::sleep(tokio::time::Duration::from_micros(2500));
        // tokio::select! {
        //     _ = s => {},
        //     _ = websocket.next() => {}
        // }
        // continue;
        // ```
        //
        // To combat this, we add a small wait to reduce how often the select loop will be woken up under high load. We
        // buffer messages to feeds so that we do as much work as possible during each wakeup, and if the
        // wakeup lasts longer than 75ms we don't wait before polling again. This knocks ~80% of a CPU worth of usage
        // off on my machine running a soak test with 500 feeds, 4 shards and 100 nodes, doesn't seem to impact
        // memory usage much, and still ensures that messages are delivered in a timely fashion.
        //
        // Increasing the wait to 100ms or more doesn't seem to have much more of a positive impact anyway.
        let debounce = tokio::time::sleep_until(tokio::time::Instant::now() + std::time::Duration::from_millis(75));

        tokio::select! {biased;

            // FRONTEND -> AGGREGATOR (relay messages to the aggregator). Biased, so messages
            // from the UI will have priority (especially important with our debounce delay).
            msg = websocket.next() => {
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

            // AGGREGATOR -> FRONTEND (buffer messages to the UI)
            msgs = rx_from_aggregator_chunks.next() => {
                // End the loop when connection from aggregator ends:
                let msgs = match msgs {
                    Some(msgs) => msgs,
                    None => break
                };

                // There is only one message type at the mo; bytes to send
                // to the websocket. collect them all up to dispatch in one shot.
                let all_ws_msgs = msgs.into_iter().map(|msg| {
                    let bytes = match msg {
                        ToFeedWebsocket::Bytes(bytes) => bytes
                    };
                    Ok(ws::Message::binary(&*bytes))
                });

                if let Err(e) = websocket.send_all(&mut futures::stream::iter(all_ws_msgs)).await {
                    log::warn!("Closing feed websocket due to error: {}", e);
                    break;
                }
            }
        }

        debounce.await;
    }

    // loop ended; give socket back to parent:
    (tx_to_aggregator, websocket)
}
