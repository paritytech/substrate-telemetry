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
    nodes: HashMap<NodeId, Node>,
    /// Best block
    best: Block,
    /// Finalized block
    finalized: Block,
    /// Block times history, stored so we can calculate averages
    block_times: NumStats<u64>,
    /// Calculated average block time
    average_block_time: Option<u64>,
    /// When the best block first arrived
    timestamp: Option<Timestamp>
}

pub enum AddNodeResult {
    Overquota,
    Added {
        chain_renamed: bool
    }
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
    pub fn new(label: Label) -> Self {
        Chain {
            labels: MostSeen::new(label),
            nodes: HashMap::new(),
            best: Block::zero(),
            finalized: Block::zero(),
            block_times: NumStats::new(50),
            average_block_time: None,
            timestamp: None
        }
    }
    /// Can we add a node? If not, it's because the chain is at its quota.
    pub fn can_add_node(&self) -> bool {
        // Dynamically determine the max nodes based on the most common
        // label so far, in case it changes to something with a different limit.
        self.nodes.len() < max_nodes(self.labels.best())
    }
    /// Assign a node to this chain. If the function returns false, it
    /// means that the node could not be added as we're at quota.
    pub fn add_node(&mut self, node_id: NodeId, node_details: NodeDetails) -> AddNodeResult {
        if !self.can_add_node() {
            return AddNodeResult::Overquota
        }

        let label_result = self.labels.insert(&node_details.chain);
        let new_node = Node::new(node_details);
        self.nodes.insert(node_id, new_node);

        AddNodeResult::Added {
            chain_renamed: label_result.has_changed()
        }
    }
    pub fn get_node(&self, node_id: NodeId) -> Option<&Node> {
        self.nodes.get(&node_id)
    }
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&node_id)
    }
    pub fn label(&self) -> &str {
        &self.labels.best()
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn nodes(&self) -> impl Iterator<Item=(NodeId, &Node)> + '_ {
        self.nodes.iter().map(|(id, node)| (*id, node))
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