use common::{
    internal_messages::{
        self,
        LocalId,
        MuteReason
    },
    node,
    util::now
};
use bimap::BiMap;
use std::{iter::FromIterator, net::Ipv4Addr, str::FromStr};
use futures::channel::{ mpsc };
use futures::{ SinkExt, StreamExt };
use std::collections::{ HashMap, HashSet };
use crate::state::{ self, State, NodeId };
use crate::feed_message::{ self, FeedMessageSerializer };
use super::find_location;

/// A unique Id is assigned per websocket connection (or more accurately,
/// per feed socket and per shard socket). This can be combined with the
/// [`LocalId`] of messages to give us a global ID.
type ConnId = u64;

/// Incoming messages come via subscriptions, and end up looking like this.
#[derive(Clone,Debug)]
pub enum ToAggregator {
    FromShardWebsocket(ConnId, FromShardWebsocket),
    FromFeedWebsocket(ConnId, FromFeedWebsocket),
    FromFindLocation(NodeId, find_location::Location)
}

/// An incoming shard connection can send these messages to the aggregator.
#[derive(Clone,Debug)]
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
        local_id: LocalId,
        reason: internal_messages::MuteReason
    }
}

/// An incoming feed connection can send these messages to the aggregator.
#[derive(Clone,Debug)]
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
#[derive(Clone,Debug)]
pub enum ToFeedWebsocket {
    Bytes(Vec<u8>)
}

/// Instances of this are responsible for handling incoming and
/// outgoing messages in the main aggregator loop.
pub struct InnerLoop {
    /// Messages from the outside world come into this:
    rx_from_external: mpsc::Receiver<ToAggregator>,

    /// The state of our chains and nodes lives here:
    node_state: State,
    /// We maintain a mapping between NodeId and ConnId+LocalId, so that we know
    /// which messages are about which nodes.
    node_ids: BiMap<NodeId, (ConnId, LocalId)>,

    /// Keep track of how to send messages out to feeds.
    feed_channels: HashMap<ConnId, mpsc::UnboundedSender<ToFeedWebsocket>>,
    /// Keep track of how to send messages out to shards.
    shard_channels: HashMap<ConnId, mpsc::Sender<ToShardWebsocket>>,

    /// Which chain is a feed subscribed to?
    feed_conn_id_to_chain: HashMap<ConnId, Box<str>>,
    /// Which feeds are subscribed to a given chain (needs to stay in sync with above)?
    chain_to_feed_conn_ids: HashMap<Box<str>, HashSet<ConnId>>,

    /// These feeds want finality info, too.
    feed_conn_id_finality: HashSet<ConnId>,

    /// Send messages here to make location requests, which are sent back into the loop.
    tx_to_locator: mpsc::UnboundedSender<(NodeId, Ipv4Addr)>
}

impl InnerLoop {
    /// Create a new inner loop handler with the various state it needs.
    pub fn new(
        rx_from_external: mpsc::Receiver<ToAggregator>,
        tx_to_locator: mpsc::UnboundedSender<(NodeId, Ipv4Addr)>,
        denylist: Vec<String>
    ) -> Self {
        InnerLoop {
            rx_from_external,
            node_state: State::new(denylist),
            node_ids: BiMap::new(),
            feed_channels: HashMap::new(),
            shard_channels: HashMap::new(),
            feed_conn_id_to_chain: HashMap::new(),
            chain_to_feed_conn_ids: HashMap::new(),
            feed_conn_id_finality: HashSet::new(),
            tx_to_locator
        }
    }

    /// Start handling and responding to incoming messages.
    pub async fn handle(mut self) {
        while let Some(msg) = self.rx_from_external.next().await {
            match msg {
                ToAggregator::FromFeedWebsocket(feed_conn_id, msg) => {
                    self.handle_from_feed(feed_conn_id, msg).await
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, msg) => {
                    self.handle_from_shard(shard_conn_id, msg).await
                },
                ToAggregator::FromFindLocation(node_id, location) => {
                    self.handle_from_find_location(node_id, location).await
                }
            }
        }
    }

    async fn handle_from_find_location(&mut self, node_id: NodeId, location: find_location::Location) {
        // TODO: Update node location here
    }

    /// Handle messages coming from shards.
    async fn handle_from_shard(&mut self, shard_conn_id: ConnId, msg: FromShardWebsocket) {
        match msg {
            FromShardWebsocket::Initialize { channel } => {
                self.shard_channels.insert(shard_conn_id, channel);
            },
            FromShardWebsocket::Add { local_id, ip, node, genesis_hash } => {
                match self.node_state.add_node(genesis_hash, node) {
                    state::AddNodeResult::ChainOnDenyList => {
                        if let Some(shard_conn) = self.shard_channels.get_mut(&shard_conn_id) {
                            let _ = shard_conn.send(ToShardWebsocket::Mute {
                                local_id,
                                reason: MuteReason::ChainNotAllowed
                            }).await;
                        }
                    },
                    state::AddNodeResult::ChainOverQuota => {
                        if let Some(shard_conn) = self.shard_channels.get_mut(&shard_conn_id) {
                            let _ = shard_conn.send(ToShardWebsocket::Mute {
                                local_id,
                                reason: MuteReason::Overquota
                            }).await;
                        }
                    },
                    state::AddNodeResult::NodeAddedToChain(details) => {
                        let node_id = details.id;
                        // Note the ID so that we know what node other messages are referring to:
                        self.node_ids.insert(node_id, (shard_conn_id, local_id));

                        let mut feed_serializer = FeedMessageSerializer::new();
                        feed_serializer.push(feed_message::AddedNode(node_id, details.node));
                        let chain_label = details.chain.label().to_owned();

                        if let Some(bytes) = feed_serializer.into_finalized() {
                            self.broadcast_to_chain_feeds(
                                &chain_label,
                                ToFeedWebsocket::Bytes(bytes)
                            ).await
                        }

                        // TODO: The node has been added. use it's IP to find a location.
                    },
                }
            },
            FromShardWebsocket::Remove { local_id } => {
                if let Some(node_id) = self.node_ids.remove_by_right(&(shard_conn_id, local_id)) {
                    // TODO: node_state.remove_node, Every feed should know about node count changes.
                }
            },
            FromShardWebsocket::Update { local_id, payload } => {
                // TODO: Fill this all in...
                let node_id = match self.node_ids.get_by_right(&(shard_conn_id, local_id)) {
                    Some(id) => id,
                    None => return
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
            FromShardWebsocket::Disconnected => {
                // The shard has disconnected; remove the shard channel, but also
                // remove any nodes associated with the shard, firing the relevant feed messages.
            }
        }
    }

    /// Handle messages coming from feeds.
    async fn handle_from_feed(&mut self, feed_conn_id: ConnId, msg: FromFeedWebsocket) {
        match msg {
            FromFeedWebsocket::Initialize { mut channel } => {
                self.feed_channels.insert(feed_conn_id, channel.clone());

                // Tell the new feed subscription some basic things to get it going:
                let mut feed_serializer = FeedMessageSerializer::new();
                feed_serializer.push(feed_message::Version(31));
                for chain in self.node_state.iter_chains() {
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
            FromFeedWebsocket::Ping { chain } => {
                let feed_channel = match self.feed_channels.get_mut(&feed_conn_id) {
                    Some(chan) => chan,
                    None => return
                };

                // Pong!
                let mut feed_serializer = FeedMessageSerializer::new();
                feed_serializer.push(feed_message::Pong(&chain));
                if let Some(bytes) = feed_serializer.into_finalized() {
                    let _ = feed_channel.send(ToFeedWebsocket::Bytes(bytes)).await;
                }
            },
            FromFeedWebsocket::Subscribe { chain } => {
                let feed_channel = match self.feed_channels.get_mut(&feed_conn_id) {
                    Some(chan) => chan,
                    None => return
                };

                // Unsubscribe from previous chain if subscribed to one:
                let old_chain_label = self.feed_conn_id_to_chain.remove(&feed_conn_id);
                if let Some(old_chain_label) = &old_chain_label {
                    if let Some(map) = self.chain_to_feed_conn_ids.get_mut(old_chain_label) {
                        map.remove(&feed_conn_id);
                    }
                }

                // Untoggle request for finality feeds:
                self.feed_conn_id_finality.remove(&feed_conn_id);

                // Get the chain we're subscribing to, ignoring the rest if it doesn't exist.
                let chain = match self.node_state.get_chain_by_label(&chain) {
                    Some(chain) => chain,
                    None => return
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
                for (idx, (gid, node)) in chain.nodes().enumerate() {
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
                self.feed_conn_id_to_chain.insert(feed_conn_id, chain.label().into());
                self.chain_to_feed_conn_ids.entry(chain.label().into()).or_default().insert(feed_conn_id);
            },
            FromFeedWebsocket::SendFinality => {
                self.feed_conn_id_finality.insert(feed_conn_id);
            },
            FromFeedWebsocket::NoMoreFinality => {
                self.feed_conn_id_finality.remove(&feed_conn_id);
            },
            FromFeedWebsocket::Disconnected => {
                // The feed has disconnected; clean up references to it:
                if let Some(chain) = self.feed_conn_id_to_chain.remove(&feed_conn_id) {
                    self.chain_to_feed_conn_ids.remove(&chain);
                }
                self.feed_channels.remove(&feed_conn_id);
                self.feed_conn_id_finality.remove(&feed_conn_id);
            },
        }
    }

    /// Send a message to all chain feeds.
    async fn broadcast_to_chain_feeds(&mut self, chain: &str, message: ToFeedWebsocket) {
        if let Some(feeds) = self.chain_to_feed_conn_ids.get(chain) {
            for &feed_id in feeds {
                // How much faster would it be if we processed these in parallel?
                if let Some(chan) = self.feed_channels.get_mut(&feed_id) {
                    chan.send(message.clone()).await;
                }
            }
        }
    }
}