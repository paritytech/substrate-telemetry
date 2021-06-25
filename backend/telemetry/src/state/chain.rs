use std::sync::Arc;
use std::collections::{ HashSet, HashMap };
use common::types::{ BlockHash };
use common::types::{Block, NodeDetails, NodeLocation, Timestamp};
use common::util::{now, DenseMap, NumStats};
use common::most_seen::{ MostSeen, self };
use common::node::Payload;
use std::iter::IntoIterator;
use once_cell::sync::Lazy;

use super::node::Node;
use super::NodeId;

pub type ChainId = usize;
pub type Label = Box<str>;

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
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp.unwrap_or(0)
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