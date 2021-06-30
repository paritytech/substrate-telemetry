use std::collections::{ HashSet };
use common::types::{ BlockHash, BlockNumber };
use common::types::{Block, Timestamp};
use common::util::{now, DenseMap, NumStats};
use common::most_seen::MostSeen;
use common::node::Payload;
use once_cell::sync::Lazy;

use crate::feed_message;

use super::node::Node;
use super::NodeId;

pub type Label = Box<str>;

const STALE_TIMEOUT: u64 = 2 * 60 * 1000; // 2 minutes

pub struct Chain {
    /// Labels that nodes use for this chain. We keep track of
    /// the most commonly used label as nodes are added/removed.
    labels: MostSeen<Label>,
    /// Set of nodes that are in this chain
    node_ids: HashSet<NodeId>,
    /// Best block
    best: Block,
    /// Finalized block
    finalized: Block,
    /// Block times history, stored so we can calculate averages
    block_times: NumStats<u64>,
    /// Calculated average block time
    average_block_time: Option<u64>,
    /// When the best block first arrived
    timestamp: Option<Timestamp>,
    /// Genesis hash of this chain
    genesis_hash: BlockHash
}

pub enum AddNodeResult {
    Overquota,
    Added {
        chain_renamed: bool
    }
}

pub struct RemoveNodeResult {
    pub chain_renamed: bool
}

/// When we update a node, we subscribe and receive various messages
/// about the update that take this form. The expectation is that we'll
/// push and broadcast these messages to feeds. This allows the caller to
/// retain control over exactly how/when that will happen.
pub enum OnUpdateNode<'a> {
    StaleNode(feed_message::StaleNode),
    BestBlock(feed_message::BestBlock),
    BestFinalized(feed_message::BestFinalized),
    ImportedBlock(feed_message::ImportedBlock<'a>),
    Hardware(feed_message::Hardware<'a>),
    NodeStatsUpdate(feed_message::NodeStatsUpdate<'a>),
    NodeIOUpdate(feed_message::NodeIOUpdate<'a>),
    AfgFinalized(feed_message::AfgFinalized),
    AfgReceivedPrecommit(feed_message::AfgReceivedPrecommit),
    AfgReceivedPrevote(feed_message::AfgReceivedPrevote),
    FinalizedBlock(feed_message::FinalizedBlock)
}

macro_rules! into_on_update {
    ($name:ident) => {
        impl <'a> From<feed_message::$name> for OnUpdateNode<'a> {
            fn from(val: feed_message::$name) -> Self {
                OnUpdateNode::$name(val)
            }
        }
    }
}
macro_rules! into_on_update_lt {
    ($name:ident) => {
        impl <'a> From<feed_message::$name<'a>> for OnUpdateNode<'a> {
            fn from(val: feed_message::$name<'a>) -> Self {
                OnUpdateNode::$name(val)
            }
        }
    }
}
into_on_update!(StaleNode);
into_on_update!(BestBlock);
into_on_update!(BestFinalized);
into_on_update_lt!(ImportedBlock);
into_on_update_lt!(Hardware);
into_on_update_lt!(NodeStatsUpdate);
into_on_update_lt!(NodeIOUpdate);
into_on_update!(AfgFinalized);
into_on_update!(AfgReceivedPrecommit);
into_on_update!(AfgReceivedPrevote);
into_on_update!(FinalizedBlock);

/// Labels of chains we consider "first party". These chains allow any
/// number of nodes to connect.
static FIRST_PARTY_NETWORKS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("Polkadot");
    set.insert("Kusama");
    set.insert("Westend");
    set.insert("Rococo");
    set
});

/// Max number of nodes allowed to connect to the telemetry server.
const THIRD_PARTY_NETWORKS_MAX_NODES: usize = 500;

impl Chain {
    /// Create a new chain with an initial label.
    pub fn new(genesis_hash: BlockHash) -> Self {
        Chain {
            labels: MostSeen::default(),
            node_ids: HashSet::new(),
            best: Block::zero(),
            finalized: Block::zero(),
            block_times: NumStats::new(50),
            average_block_time: None,
            timestamp: None,
            genesis_hash
        }
    }

    /// Can we add a node? If not, it's because the chain is at its quota.
    pub fn can_add_node(&self) -> bool {
        // Dynamically determine the max nodes based on the most common
        // label so far, in case it changes to something with a different limit.
        self.node_ids.len() < max_nodes(self.labels.best())
    }

    /// Assign a node to this chain. If the function returns false, it
    /// means that the node could not be added as we're at quota.
    pub fn add_node(&mut self, node_id: NodeId, chain_label: &Box<str>) -> AddNodeResult {
        if !self.can_add_node() {
            return AddNodeResult::Overquota
        }

        let label_result = self.labels.insert(chain_label);
        self.node_ids.insert(node_id);

        AddNodeResult::Added {
            chain_renamed: label_result.has_changed()
        }
    }

    /// Remove a node from this chain. We expect the label it used for the chain so
    /// that we can keep track of which label is most popular.
    pub fn remove_node(&mut self, node_id: NodeId, chain_label: &Box<str>) -> RemoveNodeResult {
        let label_result = self.labels.remove(&chain_label);
        self.node_ids.remove(&node_id);

        RemoveNodeResult {
            chain_renamed: label_result.has_changed()
        }
    }

    /// Attempt to update the best block seen in this chain.
    pub fn update_node<OnUpdate>(&mut self, all_nodes: &mut DenseMap<Node>, nid: NodeId, payload: Payload, mut on_update: OnUpdate)
    where OnUpdate: FnMut(OnUpdateNode)
    {
        if let Some(block) = payload.best_block() {
            self.handle_block(all_nodes, block, nid, &mut on_update);
        }

        if let Some(node) = all_nodes.get_mut(nid) {
            match payload {
                Payload::SystemInterval(ref interval) => {
                    if node.update_hardware(interval) {
                        on_update(feed_message::Hardware(nid, node.hardware()).into());
                    }

                    if let Some(stats) = node.update_stats(interval) {
                        on_update(feed_message::NodeStatsUpdate(nid, stats).into());
                    }

                    if let Some(io) = node.update_io(interval) {
                        on_update(feed_message::NodeIOUpdate(nid, io).into());
                    }
                }
                Payload::AfgAuthoritySet(authority) => {
                    node.set_validator_address(authority.authority_id.clone());
                    return;
                }
                Payload::AfgFinalized(finalized) => {
                    if let Ok(finalized_number) = finalized.finalized_number.parse::<BlockNumber>()
                    {
                        if let Some(addr) = node.details().validator.clone() {
                            on_update(feed_message::AfgFinalized(
                                addr,
                                finalized_number,
                                finalized.finalized_hash,
                            ).into());
                        }
                    }
                    return;
                }
                Payload::AfgReceivedPrecommit(precommit) => {
                    if let Ok(finalized_number) =
                        precommit.target_number.parse::<BlockNumber>()
                    {
                        if let Some(addr) = node.details().validator.clone() {
                            let voter = precommit.voter.clone();
                            on_update(feed_message::AfgReceivedPrecommit(
                                addr,
                                finalized_number,
                                precommit.target_hash,
                                voter,
                            ).into());
                        }
                    }
                    return;
                }
                Payload::AfgReceivedPrevote(prevote) => {
                    if let Ok(finalized_number) =
                        prevote.target_number.parse::<BlockNumber>()
                    {
                        if let Some(addr) = node.details().validator.clone() {
                            let voter = prevote.voter.clone();
                            on_update(feed_message::AfgReceivedPrevote(
                                addr,
                                finalized_number,
                                prevote.target_hash,
                                voter,
                            ).into());
                        }
                    }
                    return;
                }
                Payload::AfgReceivedCommit(_) => {}
                _ => (),
            }

            if let Some(block) = payload.finalized_block() {
                if let Some(finalized) = node.update_finalized(block) {
                    on_update(feed_message::FinalizedBlock(
                        nid,
                        finalized.height,
                        finalized.hash,
                    ).into());

                    if finalized.height > self.finalized.height {
                        self.finalized = *finalized;
                        on_update(feed_message::BestFinalized(finalized.height, finalized.hash).into());
                    }
                }
            }
        }
    }

    fn handle_block<OnUpdate>(&mut self, all_nodes: &mut DenseMap<Node>, block: &Block, nid: NodeId, mut on_update: OnUpdate)
    where OnUpdate: FnMut(OnUpdateNode)
    {
        let mut propagation_time = None;
        let now = now();
        let nodes_len = self.node_ids.len();

        self.update_stale_nodes(all_nodes, now, &mut on_update);

        let node = match all_nodes.get_mut(nid) {
            Some(node) => node,
            None => return,
        };

        if node.update_block(*block) {
            if block.height > self.best.height {
                self.best = *block;
                log::debug!(
                    "[{}] [nodes={}] new best block={}/{:?}",
                    self.labels.best(),
                    nodes_len,
                    self.best.height,
                    self.best.hash,
                );
                if let Some(timestamp) = self.timestamp {
                    self.block_times.push(now - timestamp);
                    self.average_block_time = Some(self.block_times.average());
                }
                self.timestamp = Some(now);
                on_update(feed_message::BestBlock(
                    self.best.height,
                    now,
                    self.average_block_time,
                ).into());
                propagation_time = Some(0);
            } else if block.height == self.best.height {
                if let Some(timestamp) = self.timestamp {
                    propagation_time = Some(now - timestamp);
                }
            }

            if let Some(details) = node.update_details(now, propagation_time) {
                on_update(feed_message::ImportedBlock(nid, details).into());
            }
        }
    }

    /// Check if the chain is stale (has not received a new best block in a while).
    /// If so, find a new best block, ignoring any stale nodes and marking them as such.
    fn update_stale_nodes<OnUpdate>(&mut self, all_nodes: &mut DenseMap<Node>, now: u64, mut on_update: OnUpdate)
    where OnUpdate: FnMut(OnUpdateNode)
    {
        let threshold = now - STALE_TIMEOUT;
        let timestamp = match self.timestamp {
            Some(ts) => ts,
            None => return,
        };

        if timestamp > threshold {
            // Timestamp is in range, nothing to do
            return;
        }

        let mut best = Block::zero();
        let mut finalized = Block::zero();
        let mut timestamp = None;

        for &nid in self.node_ids.iter() {
            let node = match all_nodes.get_mut(nid) {
                Some(node) => node,
                None => continue
            };
            if !node.update_stale(threshold) {
                if node.best().height > best.height {
                    best = *node.best();
                    timestamp = Some(node.best_timestamp());
                }

                if node.finalized().height > finalized.height {
                    finalized = *node.finalized();
                }
            } else {
                on_update(feed_message::StaleNode(nid).into());
            }
        }

        if self.best.height != 0 || self.finalized.height != 0 {
            self.best = best;
            self.finalized = finalized;
            self.block_times.reset();
            self.timestamp = timestamp;

            on_update(feed_message::BestBlock(
                self.best.height,
                timestamp.unwrap_or(now),
                None,
            ).into());
            on_update(feed_message::BestFinalized(finalized.height, finalized.hash).into());
        }
    }

    pub fn label(&self) -> &str {
        &self.labels.best()
    }
    pub fn node_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.node_ids.iter().copied()
    }
    pub fn node_count(&self) -> usize {
        self.node_ids.len()
    }
    pub fn best_block(&self) -> &Block {
        &self.best
    }
    pub fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp
    }
    pub fn average_block_time(&self) -> Option<u64> {
        self.average_block_time
    }
    pub fn finalized_block(&self) -> &Block {
        &self.finalized
    }
    pub fn genesis_hash(&self) -> &BlockHash {
        &self.genesis_hash
    }
}

/// First party networks (Polkadot, Kusama etc) are allowed any number of nodes.
/// Third party networks are allowed `THIRD_PARTY_NETWORKS_MAX_NODES` nodes and
/// no more.
fn max_nodes(label: &str) -> usize {
    if FIRST_PARTY_NETWORKS.contains(label) {
        usize::MAX
    } else {
        THIRD_PARTY_NETWORKS_MAX_NODES
    }
}