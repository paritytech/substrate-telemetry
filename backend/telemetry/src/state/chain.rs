use std::sync::Arc;
use std::collections::{ HashSet, HashMap };
use common::types::{ BlockHash };
use common::internal_messages::{ GlobalId };
use super::node::Node;
use common::types::{Block, NodeDetails, NodeId, NodeLocation, Timestamp};
use common::util::{now, DenseMap, NumStats};
use common::node::Payload;
use std::iter::IntoIterator;

pub type ChainId = usize;
pub type Label = Arc<str>;

pub struct Chain {
    /// Label of this chain, along with count of nodes that use this label
    label: (Label, usize),
    /// Chain genesis hash
    genesis_hash: BlockHash,
    /// Set of nodes that are in this chain
    nodes: HashSet<GlobalId>,
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
    /// Some nodes might manifest a different label, note them here
    labels: HashMap<Label, usize>,
    /// How many nodes are allowed in this chain
    max_nodes: usize
}

impl Chain {
    pub fn label(&self) -> &str {
        &self.label.0
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn node_ids(&self) -> impl Iterator<Item=GlobalId> + '_ {
        self.nodes.iter().copied()
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