use common::{
    internal_messages::{GlobalId, LocalId},
    node,
    assign_id::AssignId,
    util::now
};
use std::{str::FromStr, sync::Arc};
use std::sync::atomic::AtomicU64;
use futures::channel::{ mpsc, oneshot };
use futures::{ Sink, SinkExt, StreamExt };
use tokio::net::TcpStream;
use tokio_util::compat::{ TokioAsyncReadCompatExt };
use std::collections::{ HashMap, HashSet };
use crate::state::State;
use crate::feed_message::{ self, FeedMessageSerializer };

/// A unique Id is assigned per websocket connection (or more accurately,
/// per feed socket and per shard socket). This can be combined with the
/// [`LocalId`] of messages to give us a global ID.
type ConnId = u64;

/// Incoming messages come via subscriptions, and end up looking like this.
#[derive(Debug)]
enum ToAggregator {
    FromShardWebsocket(ConnId, FromShardWebsocket),
    FromFeedWebsocket(ConnId, FromFeedWebsocket),
}

/// An incoming shard connection can send these messages to the aggregator.
#[derive(Debug)]
pub enum FromShardWebsocket {
    /// When the socket is opened, it'll send this first
    /// so that we have a way to communicate back to it.
    Initialize {
        channel: mpsc::Sender<ToShardWebsocket>,
    },
    /// Tell the aggregator about a new node.
    Add {
        local_id: LocalId,
        ip: Option<std::net::IpAddr>,
        node: common::types::NodeDetails,
        genesis_hash: common::types::BlockHash
    },
    /// Update/pass through details about a node.
    Update {
        local_id: LocalId,
        payload: node::Payload
    },
    /// Tell the aggregator that a node has been removed when it disconnects.
    Remove {
        local_id: LocalId,
    },
    /// The shard is disconnected.
    Disconnected
}

/// The aggregator can these messages back to a shard connection.
#[derive(Debug)]
pub enum ToShardWebsocket {
    /// Mute messages to the core by passing the shard-local ID of them.
    Mute {
        local_id: LocalId
    }
}

/// An incoming feed connection can send these messages to the aggregator.
#[derive(Debug)]
pub enum FromFeedWebsocket {
    /// When the socket is opened, it'll send this first
    /// so that we have a way to communicate back to it.
    /// Unbounded so that slow feeds don't block aggregato
    /// progress.
    Initialize {
        channel: mpsc::UnboundedSender<ToFeedWebsocket>,
    },
    /// The feed can subscribe to a chain to receive
    /// messages relating to it.
    Subscribe {
        chain: Box<str>
    },
    /// The feed wants finality info for the chain, too.
    SendFinality,
    /// The feed doesn't want any more finality info for the chain.
    NoMoreFinality,
    /// An explicit ping message.
    Ping {
        chain: Box<str>
    },
    /// The feed is disconnected.
    Disconnected
}

// The frontend sends text based commands; parse them into these messages:
impl FromStr for FromFeedWebsocket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cmd, chain) = match s.find(':') {
            Some(idx) => (&s[..idx], s[idx+1..].into()),
            None => return Err(anyhow::anyhow!("Expecting format `CMD:CHAIN_NAME`"))
        };
        match cmd {
            "ping" => Ok(FromFeedWebsocket::Ping { chain }),
            "subscribe" => Ok(FromFeedWebsocket::Subscribe { chain }),
            "send-finality" => Ok(FromFeedWebsocket::SendFinality),
            "no-more-finality" => Ok(FromFeedWebsocket::NoMoreFinality),
            _ => return Err(anyhow::anyhow!("Command {} not recognised", cmd))
        }
    }
}

/// The aggregator can these messages back to a feed connection.
#[derive(Debug)]
pub enum ToFeedWebsocket {
    Bytes(Vec<u8>)
}

#[derive(Clone)]
pub struct Aggregator(Arc<AggregatorInternal>);

struct AggregatorInternal {
    /// Shards that connect are each assigned a unique connection ID.
    /// This helps us know who to send messages back to (especially in
    /// conjunction with the [`LocalId`] that messages will come with).
    shard_conn_id: AtomicU64,
    /// Feeds that connect have their own unique connection ID, too.
    feed_conn_id: AtomicU64,
    /// Send messages in to the aggregator from the outside via this. This is
    /// stored here so that anybody holding an `Aggregator` handle can
    /// make use of it.
    tx_to_aggregator: mpsc::Sender<ToAggregator>
}

impl Aggregator {
    /// Spawn a new Aggregator. This connects to the telemetry backend
    pub async fn spawn(denylist: Vec<String>) -> anyhow::Result<Aggregator> {
        let (tx_to_aggregator, rx_from_external) = mpsc::channel(10);

        // Handle any incoming messages in our handler loop:
        tokio::spawn(Aggregator::handle_messages(rx_from_external, denylist));

        // Return a handle to our aggregator:
        Ok(Aggregator(Arc::new(AggregatorInternal {
            shard_conn_id: AtomicU64::new(1),
            feed_conn_id: AtomicU64::new(1),
            tx_to_aggregator,
        })))
    }

    // This is spawned into a separate task and handles any messages coming
    // in to the aggregator. If nobody is tolding the tx side of the channel
    // any more, this task will gracefully end.
    async fn handle_messages(mut rx_from_external: mpsc::Receiver<ToAggregator>, denylist: Vec<String>) {

        let mut node_state = State::new(denylist);

        // Maintain mappings from the shard connection ID and local ID of messages to a global ID
        // that uniquely identifies nodes in our node state.
        let mut to_global_node_id = AssignId::new();

        // Keep track of channels to communicate with feeds and shards:
        let mut feed_channels = HashMap::new();
        let mut shard_channels = HashMap::new();

        // What chains have our feeds subscribed to (one at a time at the mo)?
        // Both of these need to be kept in sync (should move to own struct eventually).
        let mut feed_conn_id_to_chain: HashMap<ConnId, Box<str>> = HashMap::new();
        let mut chain_to_feed_conn_ids: HashMap<Box<str>, HashSet<ConnId>> = HashMap::new();

        // Which feeds want finality info too?
        let mut feed_conn_id_finality: HashSet<ConnId> = HashSet::new();

        // Now, loop and receive messages to handle.
        while let Some(msg) = rx_from_external.next().await {
            match msg {
                // FROM FEED
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Initialize { mut channel }) => {
                    feed_channels.insert(feed_conn_id, channel.clone());

                    // Tell the new feed subscription some basic things to get it going:
                    let mut feed_serializer = FeedMessageSerializer::new();
                    feed_serializer.push(feed_message::Version(31));
                    for chain in node_state.iter_chains() {
                        feed_serializer.push(feed_message::AddedChain(
                            chain.label(),
                            chain.node_count()
                        ));
                    }

                    // Send this to the channel that subscribed:
                    if let Some(bytes) = feed_serializer.into_finalized() {
                        let _ = channel.send(ToFeedWebsocket::Bytes(bytes)).await;
                    }
                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Ping { chain }) => {
                    let feed_channel = match feed_channels.get_mut(&feed_conn_id) {
                        Some(chan) => chan,
                        None => continue
                    };

                    // Pong!
                    let mut feed_serializer = FeedMessageSerializer::new();
                    feed_serializer.push(feed_message::Pong(&chain));
                    if let Some(bytes) = feed_serializer.into_finalized() {
                        let _ = feed_channel.send(ToFeedWebsocket::Bytes(bytes)).await;
                    }
                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Subscribe { chain }) => {
                    let feed_channel = match feed_channels.get_mut(&feed_conn_id) {
                        Some(chan) => chan,
                        None => continue
                    };

                    // Unsubscribe from previous chain if subscribed to one:
                    let old_chain_label = feed_conn_id_to_chain.remove(&feed_conn_id);
                    if let Some(old_chain_label) = &old_chain_label {
                        if let Some(map) = chain_to_feed_conn_ids.get_mut(old_chain_label) {
                            map.remove(&feed_conn_id);
                        }
                    }

                    // Untoggle request for finality feeds:
                    feed_conn_id_finality.remove(&feed_conn_id);

                    // Get the chain we're subscribing to, ignoring the rest if it doesn't exist.
                    let chain = match node_state.get_chain_by_label(&chain) {
                        Some(chain) => chain,
                        None => continue
                    };

                    // Send messages to the feed about the new chain:
                    let mut feed_serializer = FeedMessageSerializer::new();
                    if let Some(old_chain_label) = old_chain_label {
                        feed_serializer.push(feed_message::UnsubscribedFrom(&old_chain_label));
                    }
                    feed_serializer.push(feed_message::SubscribedTo(chain.label()));
                    feed_serializer.push(feed_message::TimeSync(now()));
                    feed_serializer.push(feed_message::BestBlock (
                        chain.best_block().height,
                        chain.timestamp(),
                        chain.average_block_time()
                    ));
                    feed_serializer.push(feed_message::BestFinalized (
                        chain.finalized_block().height,
                        chain.finalized_block().hash
                    ));
                    for (idx, (gid, node)) in node_state.get_nodes_in_chain(chain).enumerate() {
                        // Send subscription confirmation and chain head before doing all the nodes,
                        // and continue sending batches of 32 nodes a time over the wire subsequently
                        if idx % 32 == 0 {
                            if let Some(bytes) = feed_serializer.finalize() {
                                let _ = feed_channel.send(ToFeedWebsocket::Bytes(bytes)).await;
                            }
                        }
                        feed_serializer.push(feed_message::AddedNode(gid, node));
                        feed_serializer.push(feed_message::FinalizedBlock(
                            gid,
                            node.finalized().height,
                            node.finalized().hash,
                        ));
                        if node.stale() {
                            feed_serializer.push(feed_message::StaleNode(gid));
                        }
                    }
                    if let Some(bytes) = feed_serializer.into_finalized() {
                        let _ = feed_channel.send(ToFeedWebsocket::Bytes(bytes)).await;
                    }

                    // Actually make a note of the new chain subsciption:
                    feed_conn_id_to_chain.insert(feed_conn_id, chain.label().into());
                    chain_to_feed_conn_ids.entry(chain.label().into()).or_default().insert(feed_conn_id);
                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::SendFinality) => {
                    feed_conn_id_finality.insert(feed_conn_id);
                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::NoMoreFinality) => {
                    feed_conn_id_finality.remove(&feed_conn_id);
                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Disconnected) => {
                    // The feed has disconnected; clean up references to it:
                    if let Some(chain) = feed_conn_id_to_chain.remove(&feed_conn_id) {
                        chain_to_feed_conn_ids.remove(&chain);
                    }
                    feed_channels.remove(&feed_conn_id);
                    feed_conn_id_finality.remove(&feed_conn_id);
                },

                // FROM SHARD
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Initialize { channel }) => {
                    shard_channels.insert(shard_conn_id, channel);
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Add { local_id, ip, node, genesis_hash }) => {
                    // Get globalId from add_node and store that against shard/local_id.

                    // TODO: node_state.add_node. Every feed should know about node count changes.
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Remove { local_id }) => {
                    if let Some(id) = to_global_node_id.remove_by_details(&(shard_conn_id, local_id)) {
                        // TODO: node_state.remove_node, Every feed should know about node count changes.
                    }
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Update { local_id, payload }) => {
                    // TODO: Fill this all in...
                    let global_node_id = match to_global_node_id.get_id(&(shard_conn_id, local_id)) {
                        Some(id) => id,
                        None => continue
                    };

                    if let Some(block) = payload.best_block() {

                    }

                    match payload {
                        node::Payload::SystemInterval(system_interval) => {

                        },
                        node::Payload::AfgAuthoritySet(_) => {

                        },
                        node::Payload::AfgFinalized(_) => {

                        },
                        node::Payload::AfgReceivedPrecommit(_) => {

                        },
                        node::Payload::AfgReceivedPrevote(_) => {

                        },
                        // This message should have been handled before the payload made it this far:
                        node::Payload::SystemConnected(_) => {
                            unreachable!("SystemConnected message seen in Telemetry Core, but should have been handled in shard");
                        },
                        // The following messages aren't handled at the moment. List them explicitly so
                        // that we have to make an explicit choice for any new messages:
                        node::Payload::BlockImport(_) |
                        node::Payload::NotifyFinalized(_) |
                        node::Payload::AfgReceivedCommit(_) |
                        node::Payload::TxPoolImport |
                        node::Payload::AfgFinalizedBlocksUpTo |
                        node::Payload::AuraPreSealedBlock |
                        node::Payload::PreparedBlockForProposing => {},
                    }

                    // TODO: node_state.update_node, then handle returned diffs
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Disconnected) => {
                    // The shard has disconnected; remove the shard channel, but also
                    // remove any nodes associated with the shard, firing the relevant feed messages.
                }
            }
        }
    }

    /// Return a sink that a shard can send messages into to be handled by the aggregator.
    pub fn subscribe_shard(&self) -> impl Sink<FromShardWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let shard_conn_id: ConnId = self.0.shard_conn_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.with(move |msg| async move {
            Ok(ToAggregator::FromShardWebsocket(shard_conn_id, msg))
        }))
    }

    /// Return a sink that a feed can send messages into to be handled by the aggregator.
    pub fn subscribe_feed(&self) -> impl Sink<FromFeedWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let feed_conn_id: ConnId = self.0.feed_conn_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.with(move |msg| async move {
            Ok(ToAggregator::FromFeedWebsocket(feed_conn_id, msg))
        }))
    }

}