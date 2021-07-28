use crate::connection::{create_ws_connection_to_core, Message};
use common::{
    internal_messages::{self, ShardNodeId},
    node_message,
    node_types::BlockHash,
    AssignId,
};
use futures::{channel::mpsc};
use futures::{Sink, SinkExt, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// A unique Id is assigned per websocket connection (or more accurately,
/// per thing-that-subscribes-to-the-aggregator). That connection might send
/// data on behalf of multiple chains, so this ID is local to the aggregator,
/// and a unique ID is assigned per batch of data too ([`internal_messages::LocalId`]).
type ConnId = u64;

/// Incoming messages are either from websocket connections or
/// from the telemetry core. This can be private since the only
/// external messages are via subscriptions that take
/// [`FromWebsocket`] instances.
#[derive(Clone, Debug)]
enum ToAggregator {
    /// Sent when the telemetry core is disconnected.
    DisconnectedFromTelemetryCore,
    /// Sent when the telemetry core (re)connects.
    ConnectedToTelemetryCore,
    /// Sent when a message comes in from a substrate node.
    FromWebsocket(ConnId, FromWebsocket),
    /// Send when a message comes in from the telemetry core.
    FromTelemetryCore(internal_messages::FromTelemetryCore),
}

/// An incoming socket connection can provide these messages.
/// Until a node has been Added via [`FromWebsocket::Add`],
/// messages from it will be ignored.
#[derive(Clone, Debug)]
pub enum FromWebsocket {
    /// Fire this when the connection is established.
    Initialize {
        /// When a message is sent back up this channel, we terminate
        /// the websocket connection and force the node to reconnect
        /// so that it sends its system info again incase the telemetry
        /// core has restarted.
        close_connection: mpsc::Sender<()>,
    },
    /// Tell the aggregator about a new node.
    Add {
        message_id: node_message::NodeMessageId,
        ip: Option<std::net::IpAddr>,
        node: common::node_types::NodeDetails,
        genesis_hash: BlockHash,
    },
    /// Update/pass through details about a node.
    Update {
        message_id: node_message::NodeMessageId,
        payload: node_message::Payload,
    },
    /// Make a note when the node disconnects.
    Disconnected,
}

pub type FromAggregator = internal_messages::FromShardAggregator;

/// The aggregator loop handles incoming messages from nodes, or from the telemetry core.
/// this is where we decide what effect messages will have.
#[derive(Clone)]
pub struct Aggregator(Arc<AggregatorInternal>);

struct AggregatorInternal {
    /// Nodes that connect are each assigned a unique connection ID. Nodes
    /// can send messages on behalf of more than one chain, and so this ID is
    /// only really used inside the Aggregator in conjunction with a per-message
    /// ID.
    conn_id: AtomicU64,
    /// Send messages to the aggregator from websockets via this. This is
    /// stored here so that anybody holding an `Aggregator` handle can
    /// make use of it.
    tx_to_aggregator: mpsc::Sender<ToAggregator>,
}

impl Aggregator {
    /// Spawn a new Aggregator. This connects to the telemetry backend
    pub async fn spawn(telemetry_uri: http::Uri) -> anyhow::Result<Aggregator> {
        let (tx_to_aggregator, rx_from_external) = mpsc::channel(10);

        // Establish a resiliant connection to the core (this retries as needed):
        let (tx_to_telemetry_core, mut rx_from_telemetry_core) =
            create_ws_connection_to_core(telemetry_uri).await;

        // Forward messages from the telemetry core into the aggregator:
        let mut tx_to_aggregator2 = tx_to_aggregator.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx_from_telemetry_core.next().await {
                let msg_to_aggregator = match msg {
                    Message::Connected => ToAggregator::ConnectedToTelemetryCore,
                    Message::Disconnected => ToAggregator::DisconnectedFromTelemetryCore,
                    Message::Data(data) => ToAggregator::FromTelemetryCore(data),
                };
                if let Err(_) = tx_to_aggregator2.send(msg_to_aggregator).await {
                    // This will close the ws channels, which themselves log messages.
                    break
                }
            }
        });

        // Start our aggregator loop, handling any incoming messages:
        tokio::spawn(Aggregator::handle_messages(
            rx_from_external,
            tx_to_telemetry_core,
        ));

        // Return a handle to our aggregator so that we can send in messages to it:
        Ok(Aggregator(Arc::new(AggregatorInternal {
            conn_id: AtomicU64::new(1),
            tx_to_aggregator,
        })))
    }

    // This is spawned into a separate task and handles any messages coming
    // in to the aggregator. If nobody is holding the tx side of the channel
    // any more, this task will gracefully end.
    async fn handle_messages(
        mut rx_from_external: mpsc::Receiver<ToAggregator>,
        mut tx_to_telemetry_core: mpsc::Sender<FromAggregator>,
    ) {
        use internal_messages::{FromShardAggregator, FromTelemetryCore};

        // Just as an optimisation, we can keep track of whether we're connected to the backend
        // or not, and ignore incoming messages while we aren't.
        let mut connected_to_telemetry_core = false;

        // A list of close channels for the currently connected substrate nodes. Send an empty
        // tuple to these to ask the connections to be closed.
        let mut close_connections: HashMap<ConnId, mpsc::Sender<()>> = HashMap::new();

        // Maintain mappings from the connection ID and node message ID to the "local ID" which we
        // broadcast to the telemetry core.
        let mut to_local_id = AssignId::new();

        // Any messages coming from nodes that have been muted are ignored:
        let mut muted: HashSet<ShardNodeId> = HashSet::new();

        // Now, loop and receive messages to handle.
        while let Some(msg) = rx_from_external.next().await {
            match msg {
                ToAggregator::ConnectedToTelemetryCore => {
                    // Take hold of the connection closers and run them all.
                    let closers = close_connections;

                    for (_, mut closer) in closers {
                        // if this fails, it probably means the connection has died already anyway.
                        let _ = closer.send(()).await;
                    }

                    // We've told everything to disconnect. Now, reset our state:
                    close_connections = HashMap::new();
                    to_local_id.clear();
                    muted.clear();

                    connected_to_telemetry_core = true;
                    log::info!("Connected to telemetry core");
                }
                ToAggregator::DisconnectedFromTelemetryCore => {
                    connected_to_telemetry_core = false;
                    log::info!("Disconnected from telemetry core");
                }
                ToAggregator::FromWebsocket(
                    conn_id,
                    FromWebsocket::Initialize { close_connection },
                ) => {
                    // We boot all connections on a reconnect-to-core to force new systemconnected
                    // messages to be sent. We could boot on muting, but need to be careful not to boot
                    // connections where we mute one set of messages it sends and not others.
                    close_connections.insert(conn_id, close_connection);
                }
                ToAggregator::FromWebsocket(
                    conn_id,
                    FromWebsocket::Add {
                        message_id,
                        ip,
                        node,
                        genesis_hash,
                    },
                ) => {
                    // Don't bother doing anything else if we're disconnected, since we'll force the
                    // node to reconnect anyway when the backend does:
                    if !connected_to_telemetry_core {
                        continue;
                    }

                    // Generate a new "local ID" for messages from this connection:
                    let local_id = to_local_id.assign_id((conn_id, message_id));

                    // Send the message to the telemetry core with this local ID:
                    let _ = tx_to_telemetry_core
                        .send(FromShardAggregator::AddNode {
                            ip,
                            node,
                            genesis_hash,
                            local_id,
                        })
                        .await;
                }
                ToAggregator::FromWebsocket(
                    conn_id,
                    FromWebsocket::Update {
                        message_id,
                        payload,
                    },
                ) => {
                    // Ignore incoming messages if we're not connected to the backend:
                    if !connected_to_telemetry_core {
                        continue;
                    }

                    // Get the local ID, ignoring the message if none match:
                    let local_id = match to_local_id.get_id(&(conn_id, message_id)) {
                        Some(id) => id,
                        None => continue,
                    };

                    // ignore the message if this node has been muted:
                    if muted.contains(&local_id) {
                        continue;
                    }

                    // Send the message to the telemetry core with this local ID:
                    let _ = tx_to_telemetry_core
                        .send(FromShardAggregator::UpdateNode { local_id, payload })
                        .await;
                }
                ToAggregator::FromWebsocket(disconnected_conn_id, FromWebsocket::Disconnected) => {
                    // Find all of the local IDs corresponding to the disconnected connection ID and
                    // remove them, telling Telemetry Core about them too. This could be more efficient,
                    // but the mapping isn't currently cached and it's not a super frequent op.
                    let local_ids_disconnected: Vec<_> = to_local_id
                        .iter()
                        .filter(|(_, &(conn_id, _))| disconnected_conn_id == conn_id)
                        .map(|(local_id, _)| local_id)
                        .collect();

                    close_connections.remove(&disconnected_conn_id);

                    for local_id in local_ids_disconnected {
                        to_local_id.remove_by_id(local_id);
                        muted.remove(&local_id);
                        let _ = tx_to_telemetry_core
                            .send(FromShardAggregator::RemoveNode { local_id })
                            .await;
                    }
                }
                ToAggregator::FromTelemetryCore(FromTelemetryCore::Mute {
                    local_id,
                    reason: _,
                }) => {
                    // Mute the local ID we've been told to:
                    muted.insert(local_id);
                }
            }
        }
    }

    /// Return a sink that a node can send messages into to be handled by the aggregator.
    pub fn subscribe_node(&self) -> impl Sink<FromWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let conn_id: ConnId = self
            .0
            .conn_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(
            tx_to_aggregator
                .with(move |msg| async move { Ok(ToAggregator::FromWebsocket(conn_id, msg)) }),
        )
    }
}
