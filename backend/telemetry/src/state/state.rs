use std::sync::Arc;
use std::collections::{ HashSet, HashMap };
use common::types::{ BlockHash };
use super::node::Node;
use common::types::{Block, NodeDetails, NodeLocation, Timestamp};
use common::util::{now, DenseMap, NumStats};
use common::node::Payload;
use std::iter::IntoIterator;

use super::chain::{ self, Chain };

pub type NodeId = usize;
pub type Label = Arc<str>;

/// Our state constains node and chain information
pub struct State {
    next_id: NodeId,
    chains: HashMap<BlockHash, Chain>,
    chains_by_label: HashMap<Label, BlockHash>,
    chains_by_node: HashMap<NodeId, BlockHash>,
    /// Denylist for networks we do not want to allow connecting.
    denylist: HashSet<String>,
}

/// Adding a node to a chain leads to this result:
pub enum AddNodeResult<'a> {
    /// The chain is on the "deny list", so we can't add the node
    ChainOnDenyList,
    /// The chain is over quota (too many nodes connected), so can't add the node
    ChainOverQuota,
    /// The node was added to the chain
    NodeAddedToChain(NodeAddedToChain<'a>)
}

pub struct NodeAddedToChain<'a> {
    /// The ID assigned to this node.
    pub id: NodeId,
    /// The chain the node was added to.
    pub chain: &'a Chain,
    /// The node that was added.
    pub node: &'a Node,
    /// Is this chain newly added?
    pub chain_just_added: bool,
    /// Has the chain label been updated?
    pub has_chain_label_changed: bool
}

pub struct RemoveNodeResult {
    /// How many nodes remain on the chain (0 if the chain was removed):
    chain_node_count: usize
}

impl State {
    pub fn new<T: IntoIterator<Item=String>>(denylist: T) -> State {
        State {
            next_id: 0,
            chains: HashMap::new(),
            chains_by_label: HashMap::new(),
            chains_by_node: HashMap::new(),
            denylist: denylist.into_iter().collect()
        }
    }

    pub fn iter_chains(&self) -> impl Iterator<Item=&Chain> {
        self.chains
            .iter()
            .map(|(_,chain)| chain)
    }

    pub fn get_chain_by_label(&self, label: &str) -> Option<&Chain> {
        self.chains_by_label
            .get(label)
            .and_then(|chain_id| self.chains.get(chain_id))
    }

    pub fn add_node(&mut self, genesis_hash: BlockHash, node_details: NodeDetails) -> AddNodeResult<'_> {
        if self.denylist.contains(&*node_details.chain) {
            return AddNodeResult::ChainOnDenyList;
        }

        let chain = self.chains
            .entry(genesis_hash)
            .or_insert_with(|| Chain::new(node_details.chain.clone()));

        if !chain.can_add_node() {
            return AddNodeResult::ChainOverQuota;
        }

        let node_id = self.next_id;
        self.next_id += 1;

        match chain.add_node(node_id, node_details) {
            chain::AddNodeResult::Overquota => {
                AddNodeResult::ChainOverQuota
            },
            chain::AddNodeResult::Added { chain_renamed } => {
                let node = chain.get_node(node_id).unwrap();
                AddNodeResult::NodeAddedToChain(NodeAddedToChain {
                    id: node_id,
                    chain: chain,
                    node: node,
                    chain_just_added: chain.node_count() == 1,
                    has_chain_label_changed: chain_renamed
                })
            }
        }
    }

    // /// Add a new node to our state.
    // pub fn add_node(&mut self, id: GlobalId, genesis_hash: BlockHash, node: &NodeDetails) -> AddNodeResult {
    //     if self.denylist.contains(&*node.chain) {
    //         return AddNodeResult::ChainOnDenyList;
    //     }
    //     let chain_id = self.chains.get_or_create(genesis_hash, &node.chain);

    //     return Ok(())
    // }

    // /// Remove a node from our state.
    // pub fn remove_node(&mut self, id: GlobalId) -> RemoveNodeResult {

    // }

    // /// Update a node with new data. This needs breaking down into parts so
    // /// that we can emit a useful result in each case to inform the aggregator
    // /// what messages it needs to emit.
    // pub fn update_node(&mut self, id: GlobalId, payload: Payload) {

    // }

    // fn get_or_create_chain(genesis_hash: BlockHash, chain: &str) -> ChainId {

    // }
}

