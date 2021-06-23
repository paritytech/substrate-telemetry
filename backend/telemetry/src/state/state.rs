use std::sync::Arc;
use std::collections::{ HashSet, HashMap };
use common::types::{ BlockHash };
use common::internal_messages::{ GlobalId };
use super::node::Node;
use once_cell::sync::Lazy;
use common::types::{Block, NodeDetails, NodeId, NodeLocation, Timestamp};
use common::util::{now, DenseMap, NumStats};
use common::node::Payload;
use std::iter::IntoIterator;

use super::chain::Chain;

pub type ChainId = usize;
pub type Label = Arc<str>;

/// Our state constains node and chain information
pub struct State {
    chains: DenseMap<Chain>,
    nodes: HashMap<GlobalId, Node>,
    chains_by_genesis_hash: HashMap<BlockHash, ChainId>,
    chains_by_label: HashMap<Label, ChainId>,
    /// Denylist for networks we do not want to allow connecting.
    denylist: HashSet<String>,
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

/// Adding a node to a chain leads to this result:
pub enum AddNodeResult {
    /// The chain is on the "deny list", so we can't add the node
    ChainOnDenyList,
    /// The chain is over quota (too many nodes connected), so can't add the node
    ChainOverQuota,
    /// The node was added to the chain
    NodeAddedToChain(NodeAddedToChain)
}

pub struct NodeAddedToChain {
    /// The label for the chain (which may have changed as a result of adding the node):
    chain_label: Arc<str>,
    /// Has the chain label been updated?
    has_chain_label_changed: bool,
    // How many nodes now exist in the chain?
    chain_node_count: usize
}

pub struct RemoveNodeResult {
    /// How many nodes remain on the chain (0 if the chain was removed):
    chain_node_count: usize
}

impl State {
    pub fn new<T: IntoIterator<Item=String>>(denylist: T) -> State {
        State {
            chains: DenseMap::new(),
            nodes: HashMap::new(),
            chains_by_genesis_hash: HashMap::new(),
            chains_by_label: HashMap::new(),
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
            .and_then(|chain_id| self.chains.get(*chain_id))
    }

    pub fn get_nodes_in_chain<'s>(&'s self, chain: &'s Chain) -> impl Iterator<Item=(GlobalId,&Node)> {
        chain.node_ids()
            .filter_map(move |id| self.nodes.get(&id).map(|node| (id, node)))
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