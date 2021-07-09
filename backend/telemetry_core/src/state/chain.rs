use common::node_message::Payload;
use common::node_types::{Block, Timestamp};
use common::node_types::{BlockHash, BlockNumber};
use common::{id_type, time, DenseMap, MostSeen, NumStats};
use once_cell::sync::Lazy;
use std::collections::HashSet;

use crate::feed_message::{self, FeedMessageSerializer};
use crate::find_location;

use super::node::Node;

id_type! {
    /// A Node ID that is unique to the chain it's in.
    pub struct ChainNodeId(usize)
}

pub type Label = Box<str>;

const STALE_TIMEOUT: u64 = 2 * 60 * 1000; // 2 minutes

pub struct Chain {
    /// Labels that nodes use for this chain. We keep track of
    /// the most commonly used label as nodes are added/removed.
    labels: MostSeen<Label>,
    /// Set of nodes that are in this chain
    nodes: DenseMap<ChainNodeId, Node>,
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
    genesis_hash: BlockHash,
}

pub enum AddNodeResult {
    Overquota,
    Added {
        id: ChainNodeId,
        chain_renamed: bool,
    },
}

pub struct RemoveNodeResult {
    pub chain_renamed: bool,
}

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
            nodes: DenseMap::new(),
            best: Block::zero(),
            finalized: Block::zero(),
            block_times: NumStats::new(50),
            average_block_time: None,
            timestamp: None,
            genesis_hash,
        }
    }

    /// Can we add a node? If not, it's because the chain is at its quota.
    pub fn can_add_node(&self) -> bool {
        // Dynamically determine the max nodes based on the most common
        // label so far, in case it changes to something with a different limit.
        self.nodes.len() < max_nodes(self.labels.best())
    }

    /// Assign a node to this chain.
    pub fn add_node(&mut self, node: Node) -> AddNodeResult {
        if !self.can_add_node() {
            return AddNodeResult::Overquota;
        }

        let node_chain_label = &node.details().chain;
        let label_result = self.labels.insert(node_chain_label);
        let node_id = self.nodes.add(node);

        AddNodeResult::Added {
            id: node_id,
            chain_renamed: label_result.has_changed(),
        }
    }

    /// Remove a node from this chain.
    pub fn remove_node(&mut self, node_id: ChainNodeId) -> RemoveNodeResult {
        let node = match self.nodes.remove(node_id) {
            Some(node) => node,
            None => {
                return RemoveNodeResult {
                    chain_renamed: false,
                }
            }
        };

        let node_chain_label = &node.details().chain;
        let label_result = self.labels.remove(node_chain_label);

        RemoveNodeResult {
            chain_renamed: label_result.has_changed(),
        }
    }

    /// Attempt to update the best block seen in this chain.
    /// Returns a boolean which denotes whether the output is for finalization feeds (true) or not (false).
    pub fn update_node(
        &mut self,
        nid: ChainNodeId,
        payload: Payload,
        feed: &mut FeedMessageSerializer,
    ) -> bool {
        if let Some(block) = payload.best_block() {
            self.handle_block(block, nid, feed);
        }

        if let Some(node) = self.nodes.get_mut(nid) {
            match payload {
                Payload::SystemInterval(ref interval) => {
                    if node.update_hardware(interval) {
                        feed.push(feed_message::Hardware(nid.into(), node.hardware()));
                    }

                    if let Some(stats) = node.update_stats(interval) {
                        feed.push(feed_message::NodeStatsUpdate(nid.into(), stats));
                    }

                    if let Some(io) = node.update_io(interval) {
                        feed.push(feed_message::NodeIOUpdate(nid.into(), io));
                    }
                }
                Payload::AfgAuthoritySet(authority) => {
                    node.set_validator_address(authority.authority_id.clone());
                    return false;
                }
                Payload::AfgFinalized(finalized) => {
                    if let Ok(finalized_number) = finalized.finalized_number.parse::<BlockNumber>()
                    {
                        if let Some(addr) = node.details().validator.clone() {
                            feed.push(feed_message::AfgFinalized(
                                addr,
                                finalized_number,
                                finalized.finalized_hash,
                            ));
                        }
                    }
                    return true;
                }
                Payload::AfgReceivedPrecommit(precommit) => {
                    if let Ok(finalized_number) = precommit.target_number.parse::<BlockNumber>() {
                        if let Some(addr) = node.details().validator.clone() {
                            let voter = precommit.voter.clone();
                            feed.push(feed_message::AfgReceivedPrecommit(
                                addr,
                                finalized_number,
                                precommit.target_hash,
                                voter,
                            ));
                        }
                    }
                    return true;
                }
                Payload::AfgReceivedPrevote(prevote) => {
                    if let Ok(finalized_number) = prevote.target_number.parse::<BlockNumber>() {
                        if let Some(addr) = node.details().validator.clone() {
                            let voter = prevote.voter.clone();
                            feed.push(feed_message::AfgReceivedPrevote(
                                addr,
                                finalized_number,
                                prevote.target_hash,
                                voter,
                            ));
                        }
                    }
                    return true;
                }
                Payload::AfgReceivedCommit(_) => {}
                _ => (),
            }

            if let Some(block) = payload.finalized_block() {
                if let Some(finalized) = node.update_finalized(block) {
                    feed.push(feed_message::FinalizedBlock(
                        nid.into(),
                        finalized.height,
                        finalized.hash,
                    ));

                    if finalized.height > self.finalized.height {
                        self.finalized = *finalized;
                        feed.push(feed_message::BestFinalized(
                            finalized.height,
                            finalized.hash,
                        ));
                    }
                }
            }
        }

        false
    }

    fn handle_block(&mut self, block: &Block, nid: ChainNodeId, feed: &mut FeedMessageSerializer) {
        let mut propagation_time = None;
        let now = time::now();
        let nodes_len = self.nodes.len();

        self.update_stale_nodes(now, feed);

        let node = match self.nodes.get_mut(nid) {
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
                feed.push(feed_message::BestBlock(
                    self.best.height,
                    now,
                    self.average_block_time,
                ));
                propagation_time = Some(0);
            } else if block.height == self.best.height {
                if let Some(timestamp) = self.timestamp {
                    propagation_time = Some(now - timestamp);
                }
            }

            if let Some(details) = node.update_details(now, propagation_time) {
                feed.push(feed_message::ImportedBlock(nid.into(), details));
            }
        }
    }

    /// Check if the chain is stale (has not received a new best block in a while).
    /// If so, find a new best block, ignoring any stale nodes and marking them as such.
    fn update_stale_nodes(&mut self, now: u64, feed: &mut FeedMessageSerializer) {
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

        for (nid, node) in self.nodes.iter_mut() {
            if !node.update_stale(threshold) {
                if node.best().height > best.height {
                    best = *node.best();
                    timestamp = Some(node.best_timestamp());
                }

                if node.finalized().height > finalized.height {
                    finalized = *node.finalized();
                }
            } else {
                feed.push(feed_message::StaleNode(nid.into()));
            }
        }

        if self.best.height != 0 || self.finalized.height != 0 {
            self.best = best;
            self.finalized = finalized;
            self.block_times.reset();
            self.timestamp = timestamp;

            feed.push(feed_message::BestBlock(
                self.best.height,
                timestamp.unwrap_or(now),
                None,
            ));
            feed.push(feed_message::BestFinalized(
                finalized.height,
                finalized.hash,
            ));
        }
    }

    pub fn update_node_location(
        &mut self,
        node_id: ChainNodeId,
        location: find_location::Location,
    ) -> bool {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.update_location(location);
            true
        } else {
            false
        }
    }

    pub fn get_node(&self, id: ChainNodeId) -> Option<&Node> {
        self.nodes.get(id)
    }
    pub fn iter_nodes(&self) -> impl Iterator<Item = (ChainNodeId, &Node)> {
        self.nodes.iter()
    }
    pub fn label(&self) -> &str {
        &self.labels.best()
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
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
