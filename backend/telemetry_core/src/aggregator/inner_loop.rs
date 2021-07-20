use super::aggregator::ConnId;
use crate::feed_message::{self, FeedMessageSerializer};
use crate::find_location;
use crate::state::{self, NodeId, State};
use bimap::BiMap;
use common::{
    internal_messages::{self, MuteReason, ShardNodeId},
    node_message,
    node_types::BlockHash,
    time,
};
use futures::channel::mpsc;
use futures::StreamExt;
use std::collections::{HashMap, HashSet};
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

/// Incoming messages come via subscriptions, and end up looking like this.
#[derive(Clone, Debug)]
pub enum ToAggregator {
    FromShardWebsocket(ConnId, FromShardWebsocket),
    FromFeedWebsocket(ConnId, FromFeedWebsocket),
    FromFindLocation(NodeId, find_location::Location),
}

/// An incoming shard connection can send these messages to the aggregator.
#[derive(Clone, Debug)]
pub enum FromShardWebsocket {
    /// When the socket is opened, it'll send this first
    /// so that we have a way to communicate back to it.
    Initialize {
        channel: mpsc::UnboundedSender<ToShardWebsocket>,
    },
    /// Tell the aggregator about a new node.
    Add {
        local_id: ShardNodeId,
        ip: Option<std::net::IpAddr>,
        node: common::node_types::NodeDetails,
        genesis_hash: common::node_types::BlockHash,
    },
    /// Update/pass through details about a node.
    Update {
        local_id: ShardNodeId,
        payload: node_message::Payload,
    },
    /// Tell the aggregator that a node has been removed when it disconnects.
    Remove { local_id: ShardNodeId },
    /// The shard is disconnected.
    Disconnected,
}

/// The aggregator can these messages back to a shard connection.
#[derive(Debug)]
pub enum ToShardWebsocket {
    /// Mute messages to the core by passing the shard-local ID of them.
    Mute {
        local_id: ShardNodeId,
        reason: internal_messages::MuteReason,
    },
}

/// An incoming feed connection can send these messages to the aggregator.
#[derive(Clone, Debug)]
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
    Subscribe { chain: Box<str> },
    /// The feed wants finality info for the chain, too.
    SendFinality,
    /// The feed doesn't want any more finality info for the chain.
    NoMoreFinality,
    /// An explicit ping message.
    Ping { value: Box<str> },
    /// The feed is disconnected.
    Disconnected,
}

// The frontend sends text based commands; parse them into these messages:
impl FromStr for FromFeedWebsocket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cmd, value) = match s.find(':') {
            Some(idx) => (&s[..idx], s[idx + 1..].into()),
            None => return Err(anyhow::anyhow!("Expecting format `CMD:CHAIN_NAME`")),
        };
        match cmd {
            "ping" => Ok(FromFeedWebsocket::Ping { value }),
            "subscribe" => Ok(FromFeedWebsocket::Subscribe { chain: value }),
            "send-finality" => Ok(FromFeedWebsocket::SendFinality),
            "no-more-finality" => Ok(FromFeedWebsocket::NoMoreFinality),
            _ => return Err(anyhow::anyhow!("Command {} not recognised", cmd)),
        }
    }
}

/// The aggregator can these messages back to a feed connection.
#[derive(Clone, Debug)]
pub enum ToFeedWebsocket {
    Bytes(bytes::Bytes),
}

/// Instances of this are responsible for handling incoming and
/// outgoing messages in the main aggregator loop.
pub struct InnerLoop {
    /// Messages from the outside world come into this:
    rx_from_external: mpsc::UnboundedReceiver<ToAggregator>,

    /// The state of our chains and nodes lives here:
    node_state: State,
    /// We maintain a mapping between NodeId and ConnId+LocalId, so that we know
    /// which messages are about which nodes.
    node_ids: BiMap<NodeId, (ConnId, ShardNodeId)>,

    /// Keep track of how to send messages out to feeds.
    feed_channels: HashMap<ConnId, mpsc::UnboundedSender<ToFeedWebsocket>>,
    /// Keep track of how to send messages out to shards.
    shard_channels: HashMap<ConnId, mpsc::UnboundedSender<ToShardWebsocket>>,

    /// Which chain is a feed subscribed to?
    /// Feed Connection ID -> Chain Genesis Hash
    feed_conn_id_to_chain: HashMap<ConnId, BlockHash>,
    /// Which feeds are subscribed to a given chain (needs to stay in sync with above)?
    /// Chain Genesis Hash -> Feed Connection IDs
    chain_to_feed_conn_ids: HashMap<BlockHash, HashSet<ConnId>>,

    /// These feeds want finality info, too.
    feed_conn_id_finality: HashSet<ConnId>,

    /// Send messages here to make geographical location requests.
    tx_to_locator: mpsc::UnboundedSender<(NodeId, Ipv4Addr)>,
}

impl InnerLoop {
    /// Create a new inner loop handler with the various state it needs.
    pub fn new(
        rx_from_external: mpsc::UnboundedReceiver<ToAggregator>,
        tx_to_locator: mpsc::UnboundedSender<(NodeId, Ipv4Addr)>,
        denylist: Vec<String>,
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
            tx_to_locator,
        }
    }

    /// Start handling and responding to incoming messages. Owing to unbounded channels, we actually
    /// only have a single `.await` (in this function). This helps to make it clear that the aggregator loop
    /// will be able to make progress quickly without any potential yield points.
    pub async fn handle(mut self) {
        while let Some(msg) = self.rx_from_external.next().await {
            match msg {
                ToAggregator::FromFeedWebsocket(feed_conn_id, msg) => {
                    self.handle_from_feed(feed_conn_id, msg)
                }
                ToAggregator::FromShardWebsocket(shard_conn_id, msg) => {
                    self.handle_from_shard(shard_conn_id, msg)
                }
                ToAggregator::FromFindLocation(node_id, location) => {
                    self.handle_from_find_location(node_id, location)
                }
            }
        }
    }

    /// Handle messages that come from the node geographical locator.
    fn handle_from_find_location(
        &mut self,
        node_id: NodeId,
        location: find_location::Location,
    ) {
        self.node_state
            .update_node_location(node_id, location.clone());

        if let Some(loc) = location {
            let mut feed_message_serializer = FeedMessageSerializer::new();
            feed_message_serializer.push(feed_message::LocatedNode(
                node_id.get_chain_node_id().into(),
                loc.latitude,
                loc.longitude,
                &loc.city,
            ));

            let chain_genesis_hash = self
                .node_state
                .get_chain_by_node_id(node_id)
                .map(|chain| *chain.genesis_hash());

            if let Some(chain_genesis_hash) = chain_genesis_hash {
                self.finalize_and_broadcast_to_chain_feeds(
                    &chain_genesis_hash,
                    feed_message_serializer,
                );
            }
        }
    }

    /// Handle messages coming from shards.
    fn handle_from_shard(&mut self, shard_conn_id: ConnId, msg: FromShardWebsocket) {
        log::debug!("Message from shard ({:?}): {:?}", shard_conn_id, msg);

        match msg {
            FromShardWebsocket::Initialize { channel } => {
                self.shard_channels.insert(shard_conn_id, channel);
            }
            FromShardWebsocket::Add {
                local_id,
                ip,
                node,
                genesis_hash,
            } => {
                match self.node_state.add_node(genesis_hash, node) {
                    state::AddNodeResult::ChainOnDenyList => {
                        if let Some(shard_conn) = self.shard_channels.get_mut(&shard_conn_id) {
                            let _ = shard_conn
                                .unbounded_send(ToShardWebsocket::Mute {
                                    local_id,
                                    reason: MuteReason::ChainNotAllowed,
                                });
                        }
                    }
                    state::AddNodeResult::ChainOverQuota => {
                        if let Some(shard_conn) = self.shard_channels.get_mut(&shard_conn_id) {
                            let _ = shard_conn
                                .unbounded_send(ToShardWebsocket::Mute {
                                    local_id,
                                    reason: MuteReason::Overquota,
                                });
                        }
                    }
                    state::AddNodeResult::NodeAddedToChain(details) => {
                        let node_id = details.id;

                        // Record ID <-> (shardId,localId) for future messages:
                        self.node_ids.insert(node_id, (shard_conn_id, local_id));

                        // Don't hold onto details too long because we want &mut self later:
                        let old_chain_label = details.old_chain_label.to_owned();
                        let new_chain_label = details.new_chain_label.to_owned();
                        let chain_node_count = details.chain_node_count;
                        let has_chain_label_changed = details.has_chain_label_changed;

                        // Tell chain subscribers about the node we've just added:
                        let mut feed_messages_for_chain = FeedMessageSerializer::new();
                        feed_messages_for_chain.push(feed_message::AddedNode(
                            node_id.get_chain_node_id().into(),
                            &details.node,
                        ));
                        self.finalize_and_broadcast_to_chain_feeds(
                            &genesis_hash,
                            feed_messages_for_chain,
                        );
                        // Tell everybody about the new node count and potential rename:
                        let mut feed_messages_for_all = FeedMessageSerializer::new();
                        if has_chain_label_changed {
                            feed_messages_for_all
                                .push(feed_message::RemovedChain(&old_chain_label));
                        }
                        feed_messages_for_all
                            .push(feed_message::AddedChain(&new_chain_label, chain_node_count));
                        self.finalize_and_broadcast_to_all_feeds(feed_messages_for_all);

                        // Ask for the grographical location of the node.
                        // Currently we only geographically locate IPV4 addresses so ignore IPV6.
                        if let Some(IpAddr::V4(ip_v4)) = ip {
                            let _ = self.tx_to_locator.unbounded_send((node_id, ip_v4));
                        }
                    }
                }
            }
            FromShardWebsocket::Remove { local_id } => {
                let node_id = match self.node_ids.remove_by_right(&(shard_conn_id, local_id)) {
                    Some((node_id, _)) => node_id,
                    None => {
                        log::error!(
                            "Cannot find ID for node with shard/connectionId of {:?}/{:?}",
                            shard_conn_id,
                            local_id
                        );
                        return;
                    }
                };
                self.remove_nodes_and_broadcast_result(Some(node_id));
            }
            FromShardWebsocket::Update { local_id, payload } => {
                let node_id = match self.node_ids.get_by_right(&(shard_conn_id, local_id)) {
                    Some(id) => *id,
                    None => {
                        log::error!(
                            "Cannot find ID for node with shard/connectionId of {:?}/{:?}",
                            shard_conn_id,
                            local_id
                        );
                        return;
                    }
                };

                let mut feed_message_serializer = FeedMessageSerializer::new();
                let broadcast_finality =
                    self.node_state
                        .update_node(node_id, payload, &mut feed_message_serializer);

                if let Some(chain) = self.node_state.get_chain_by_node_id(node_id) {
                    let genesis_hash = *chain.genesis_hash();
                    if broadcast_finality {
                        self.finalize_and_broadcast_to_chain_finality_feeds(
                            &genesis_hash,
                            feed_message_serializer,
                        );
                    } else {
                        self.finalize_and_broadcast_to_chain_feeds(
                            &genesis_hash,
                            feed_message_serializer,
                        );
                    }
                }
            }
            FromShardWebsocket::Disconnected => {
                // Find all nodes associated with this shard connection ID:
                let node_ids_to_remove: Vec<NodeId> = self
                    .node_ids
                    .iter()
                    .filter(|(_, &(this_shard_conn_id, _))| shard_conn_id == this_shard_conn_id)
                    .map(|(&node_id, _)| node_id)
                    .collect();

                // ... and remove them:
                self.remove_nodes_and_broadcast_result(node_ids_to_remove);
            }
        }
    }

    /// Handle messages coming from feeds.
    fn handle_from_feed(&mut self, feed_conn_id: ConnId, msg: FromFeedWebsocket) {
        log::debug!("Message from feed ({:?}): {:?}", feed_conn_id, msg);
        match msg {
            FromFeedWebsocket::Initialize { channel } => {
                self.feed_channels.insert(feed_conn_id, channel.clone());

                // Tell the new feed subscription some basic things to get it going:
                let mut feed_serializer = FeedMessageSerializer::new();
                feed_serializer.push(feed_message::Version(31));
                for chain in self.node_state.iter_chains() {
                    feed_serializer
                        .push(feed_message::AddedChain(chain.label(), chain.node_count()));
                }

                // Send this to the channel that subscribed:
                if let Some(bytes) = feed_serializer.into_finalized() {
                    let _ = channel.unbounded_send(ToFeedWebsocket::Bytes(bytes));
                }
            }
            FromFeedWebsocket::Ping { value } => {
                let feed_channel = match self.feed_channels.get_mut(&feed_conn_id) {
                    Some(chan) => chan,
                    None => return,
                };

                // Pong!
                let mut feed_serializer = FeedMessageSerializer::new();
                feed_serializer.push(feed_message::Pong(&value));
                if let Some(bytes) = feed_serializer.into_finalized() {
                    let _ = feed_channel.unbounded_send(ToFeedWebsocket::Bytes(bytes));
                }
            }
            FromFeedWebsocket::Subscribe { chain } => {
                let feed_channel = match self.feed_channels.get_mut(&feed_conn_id) {
                    Some(chan) => chan,
                    None => return,
                };

                // Unsubscribe from previous chain if subscribed to one:
                let old_genesis_hash = self.feed_conn_id_to_chain.remove(&feed_conn_id);
                if let Some(old_genesis_hash) = &old_genesis_hash {
                    if let Some(map) = self.chain_to_feed_conn_ids.get_mut(old_genesis_hash) {
                        map.remove(&feed_conn_id);
                    }
                }

                // Untoggle request for finality feeds:
                self.feed_conn_id_finality.remove(&feed_conn_id);

                // Get old chain if there was one:
                let node_state = &self.node_state;
                let old_chain =
                    old_genesis_hash.and_then(|hash| node_state.get_chain_by_genesis_hash(&hash));

                // Get new chain, ignoring the rest if it doesn't exist.
                let new_chain = match self.node_state.get_chain_by_label(&chain) {
                    Some(chain) => chain,
                    None => return,
                };

                // Send messages to the feed about this subscription:
                let mut feed_serializer = FeedMessageSerializer::new();
                if let Some(old_chain) = old_chain {
                    feed_serializer.push(feed_message::UnsubscribedFrom(old_chain.label()));
                }
                feed_serializer.push(feed_message::SubscribedTo(new_chain.label()));
                feed_serializer.push(feed_message::TimeSync(time::now()));
                feed_serializer.push(feed_message::BestBlock(
                    new_chain.best_block().height,
                    new_chain.timestamp(),
                    new_chain.average_block_time(),
                ));
                feed_serializer.push(feed_message::BestFinalized(
                    new_chain.finalized_block().height,
                    new_chain.finalized_block().hash,
                ));
                for (idx, (chain_node_id, node)) in new_chain.iter_nodes().enumerate() {
                    let chain_node_id = chain_node_id.into();

                    // Send subscription confirmation and chain head before doing all the nodes,
                    // and continue sending batches of 32 nodes a time over the wire subsequently
                    if idx % 32 == 0 {
                        if let Some(bytes) = feed_serializer.finalize() {
                            let _ = feed_channel.unbounded_send(ToFeedWebsocket::Bytes(bytes));
                        }
                    }
                    feed_serializer.push(feed_message::AddedNode(chain_node_id, node));
                    feed_serializer.push(feed_message::FinalizedBlock(
                        chain_node_id,
                        node.finalized().height,
                        node.finalized().hash,
                    ));
                    if node.stale() {
                        feed_serializer.push(feed_message::StaleNode(chain_node_id));
                    }
                }
                if let Some(bytes) = feed_serializer.into_finalized() {
                    let _ = feed_channel.unbounded_send(ToFeedWebsocket::Bytes(bytes));
                }

                // Actually make a note of the new chain subsciption:
                let new_genesis_hash = *new_chain.genesis_hash();
                self.feed_conn_id_to_chain
                    .insert(feed_conn_id, new_genesis_hash);
                self.chain_to_feed_conn_ids
                    .entry(new_genesis_hash)
                    .or_default()
                    .insert(feed_conn_id);
            }
            FromFeedWebsocket::SendFinality => {
                self.feed_conn_id_finality.insert(feed_conn_id);
            }
            FromFeedWebsocket::NoMoreFinality => {
                self.feed_conn_id_finality.remove(&feed_conn_id);
            }
            FromFeedWebsocket::Disconnected => {
                // The feed has disconnected; clean up references to it:
                if let Some(chain) = self.feed_conn_id_to_chain.remove(&feed_conn_id) {
                    self.chain_to_feed_conn_ids.remove(&chain);
                }
                self.feed_channels.remove(&feed_conn_id);
                self.feed_conn_id_finality.remove(&feed_conn_id);
            }
        }
    }

    /// Remove all of the node IDs provided and broadcast messages to feeds as needed.
    fn remove_nodes_and_broadcast_result(
        &mut self,
        node_ids: impl IntoIterator<Item = NodeId>,
    ) {
        // Group by chain to simplify the handling of feed messages:
        let mut node_ids_per_chain: HashMap<BlockHash, Vec<NodeId>> = HashMap::new();
        for node_id in node_ids.into_iter() {
            if let Some(chain) = self.node_state.get_chain_by_node_id(node_id) {
                node_ids_per_chain
                    .entry(*chain.genesis_hash())
                    .or_default()
                    .push(node_id);
            }
        }

        // Remove the nodes for each chain
        let mut feed_messages_for_all = FeedMessageSerializer::new();
        for (chain_label, node_ids) in node_ids_per_chain {
            let mut feed_messages_for_chain = FeedMessageSerializer::new();
            for node_id in node_ids {
                self.remove_node(
                    node_id,
                    &mut feed_messages_for_chain,
                    &mut feed_messages_for_all,
                );
            }
            self.finalize_and_broadcast_to_chain_feeds(&chain_label, feed_messages_for_chain);
        }
        self.finalize_and_broadcast_to_all_feeds(feed_messages_for_all);
    }

    /// Remove a single node by its ID, pushing any messages we'd want to send
    /// out to feeds onto the provided feed serializers. Doesn't actually send
    /// anything to the feeds; just updates state as needed.
    fn remove_node(
        &mut self,
        node_id: NodeId,
        feed_for_chain: &mut FeedMessageSerializer,
        feed_for_all: &mut FeedMessageSerializer,
    ) {
        // Remove our top level association (this may already have been done).
        self.node_ids.remove_by_left(&node_id);

        let removed_details = match self.node_state.remove_node(node_id) {
            Some(remove_details) => remove_details,
            None => {
                log::error!("Could not find node {:?}", node_id);
                return;
            }
        };

        // The chain has been removed (no nodes left in it, or it was renamed):
        if removed_details.chain_node_count == 0 || removed_details.has_chain_label_changed {
            feed_for_all.push(feed_message::RemovedChain(&removed_details.old_chain_label));
        }

        // If the chain still exists, tell everybody about the new label or updated node count:
        if removed_details.chain_node_count != 0 {
            feed_for_all.push(feed_message::AddedChain(
                &removed_details.new_chain_label,
                removed_details.chain_node_count,
            ));
        }

        // Assuming the chain hasn't gone away, tell chain subscribers about the node removal
        if removed_details.chain_node_count != 0 {
            feed_for_chain.push(feed_message::RemovedNode(
                node_id.get_chain_node_id().into(),
            ));
        }
    }

    /// Finalize a [`FeedMessageSerializer`] and broadcast the result to feeds for the chain.
    fn finalize_and_broadcast_to_chain_feeds(
        &mut self,
        genesis_hash: &BlockHash,
        serializer: FeedMessageSerializer,
    ) {
        if let Some(bytes) = serializer.into_finalized() {
            self.broadcast_to_chain_feeds(genesis_hash, ToFeedWebsocket::Bytes(bytes));
        }
    }

    /// Send a message to all chain feeds.
    fn broadcast_to_chain_feeds(&mut self, genesis_hash: &BlockHash, message: ToFeedWebsocket) {
        if let Some(feeds) = self.chain_to_feed_conn_ids.get(genesis_hash) {
            for &feed_id in feeds {
                if let Some(chan) = self.feed_channels.get_mut(&feed_id) {
                    let _ = chan.unbounded_send(message.clone());
                }
            }
        }
    }

    /// Finalize a [`FeedMessageSerializer`] and broadcast the result to all feeds
    fn finalize_and_broadcast_to_all_feeds(&mut self, serializer: FeedMessageSerializer) {
        if let Some(bytes) = serializer.into_finalized() {
            self.broadcast_to_all_feeds(ToFeedWebsocket::Bytes(bytes));
        }
    }

    /// Send a message to everybody.
    fn broadcast_to_all_feeds(&mut self, message: ToFeedWebsocket) {
        for chan in self.feed_channels.values_mut() {
            let _ = chan.unbounded_send(message.clone());
        }
    }

    /// Finalize a [`FeedMessageSerializer`] and broadcast the result to chain finality feeds
    fn finalize_and_broadcast_to_chain_finality_feeds(
        &mut self,
        genesis_hash: &BlockHash,
        serializer: FeedMessageSerializer,
    ) {
        if let Some(bytes) = serializer.into_finalized() {
            self.broadcast_to_chain_finality_feeds(genesis_hash, ToFeedWebsocket::Bytes(bytes));
        }
    }

    /// Send a message to all chain finality feeds.
    fn broadcast_to_chain_finality_feeds(
        &mut self,
        genesis_hash: &BlockHash,
        message: ToFeedWebsocket,
    ) {
        if let Some(feeds) = self.chain_to_feed_conn_ids.get(genesis_hash) {
            // Get all feeds for the chain, but only broadcast to those feeds that
            // are also subscribed to receive finality updates.
            for &feed_id in feeds.union(&self.feed_conn_id_finality) {
                if let Some(chan) = self.feed_channels.get_mut(&feed_id) {
                    let _ = chan.unbounded_send(message.clone());
                }
            }
        }
    }
}
