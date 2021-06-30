use std::collections::{ HashSet, HashMap };
use common::types::{ BlockHash };
use super::node::Node;
use common::types::{Block, NodeDetails, Timestamp};
use common::util::{DenseMap};
use common::node::Payload;
use std::iter::IntoIterator;
use crate::feed_message::FeedMessageSerializer;
use crate::find_location;

use super::chain::{ self, Chain };

pub type NodeId = usize;
pub type ChainId = usize;

/// Our state constains node and chain information
pub struct State {
    // Store nodes and chains in a fairly compact format.
    nodes: DenseMap<Node>,
    chains: DenseMap<Chain>,

    // Find the right chain given various details.
    chains_by_genesis_hash: HashMap<BlockHash, ChainId>,
    chains_by_label: HashMap<Box<str>, ChainId>,
    chains_by_node: HashMap<NodeId, ChainId>,

    /// Chain labels that we do not want to allow connecting.
    denylist: HashSet<String>,
}

/// Adding a node to a chain leads to this result
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
    /// The old label of the chain.
    pub old_chain_label: Box<str>,
    /// The new label of the chain.
    pub new_chain_label: &'a str,
    /// The node that was added.
    pub node: &'a Node,
    /// Number of nodes in the chain. If 1, the chain was just added.
    pub chain_node_count: usize,
    /// Has the chain label been updated?
    pub has_chain_label_changed: bool
}

/// if removing a node is successful, we get this information back.
pub struct RemovedNode {
    /// How many nodes remain on the chain (0 if the chain was removed)
    pub chain_node_count: usize,
    /// Has the chain label been updated?
    pub has_chain_label_changed: bool,
    /// The old label of the chain.
    pub old_chain_label: Box<str>,
    /// The new label of the chain.
    pub new_chain_label: Box<str>,
}

/// If removing a node goes wrong, we get this back
#[derive(Debug, thiserror::Error)]
pub enum RemoveNodeError {
    /// The node that you tried to remove wasn't found
    #[error("Node not found")]
    NodeNotFound,
    /// The chain associated to the node wasn't found
    #[error("Node chain not found")]
    NodeChainNotFound
}

impl State {
    pub fn new<T: IntoIterator<Item=String>>(denylist: T) -> State {
        State {
            nodes: DenseMap::new(),
            chains: DenseMap::new(),
            chains_by_genesis_hash: HashMap::new(),
            chains_by_label: HashMap::new(),
            chains_by_node: HashMap::new(),
            denylist: denylist.into_iter().collect(),
        }
    }

    pub fn iter_chains(&self) -> impl Iterator<Item=StateChain<'_>> {
        self.chains
            .iter()
            .map(move |(_,chain)| StateChain { state: self, chain })
    }

    pub fn get_chain_by_node_id(&self, node_id: NodeId) -> Option<StateChain<'_>> {
        self.chains_by_node
            .get(&node_id)
            .and_then(|&chain_id| self.chains.get(chain_id))
            .map(|chain| StateChain { state: self, chain })
    }

    pub fn get_chain_by_genesis_hash(&self, genesis_hash: &BlockHash) -> Option<StateChain<'_>> {
        self.chains_by_genesis_hash
            .get(genesis_hash)
            .and_then(|&chain_id| self.chains.get(chain_id))
            .map(|chain| StateChain { state: self, chain })
    }

    pub fn get_chain_by_label(&self, label: &str) -> Option<StateChain<'_>> {
        self.chains_by_label
            .get(label)
            .and_then(|&chain_id| self.chains.get(chain_id))
            .map(|chain| StateChain { state: self, chain })
    }

    pub fn add_node(&mut self, genesis_hash: BlockHash, node_details: NodeDetails) -> AddNodeResult<'_> {
        if self.denylist.contains(&*node_details.chain) {
            return AddNodeResult::ChainOnDenyList;
        }

        // Get the chain ID, creating a new empty chain if one doesn't exist.
        // If we create a chain here, we are expecting that it will allow at
        // least this node to be added, because we don't currently try and clean it up
        // if the add fails.
        let chain_id = match self.chains_by_genesis_hash.get(&genesis_hash) {
            Some(id) => *id,
            None => {
                let chain_id = self.chains.add(Chain::new(genesis_hash));
                self.chains_by_genesis_hash.insert(genesis_hash, chain_id);
                chain_id
            }
        };

        // Get the chain.
        let chain = self.chains.get_mut(chain_id)
            .expect("should be known to exist after the above (unless chains_by_genesis_hash out of sync)");

        // What ID will the node have when it's added? We don't actually want
        // to add it until we know whether the chain will accept it, but we want
        // an ID to give to the chain.
        let node_id = self.nodes.next_id();
        let node_chain_label = node_details.chain.clone();
        let old_chain_label = chain.label().into();

        match chain.add_node(node_id, &node_chain_label) {
            chain::AddNodeResult::Overquota => {
                AddNodeResult::ChainOverQuota
            },
            chain::AddNodeResult::Added { chain_renamed } => {
                let chain = &*chain;

                // Actually add the node, and a reference to its chain,
                // if the chain adds it successfully:
                self.nodes.add(Node::new(node_details));
                self.chains_by_node.insert(node_id, chain_id);

                // Update the label we use to reference the chain if
                // it changes (it'll always change first time a node's added):
                if chain_renamed {
                    self.chains_by_label.remove(&old_chain_label);
                    self.chains_by_label.insert(chain.label().to_string().into_boxed_str(), chain_id);
                }

                let node = self.nodes.get(node_id).expect("node added above");
                AddNodeResult::NodeAddedToChain(NodeAddedToChain {
                    id: node_id,
                    node: node,
                    old_chain_label: old_chain_label,
                    new_chain_label: chain.label(),
                    chain_node_count: chain.node_count(),
                    has_chain_label_changed: chain_renamed
                })
            }
        }
    }

    /// Remove a node
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<RemovedNode,RemoveNodeError> {
        self.nodes.remove(node_id)
            .ok_or(RemoveNodeError::NodeNotFound)?;

        let chain_id = self.chains_by_node.remove(&node_id)
            .ok_or(RemoveNodeError::NodeChainNotFound)?;

        let chain = self.chains.get_mut(chain_id)
            .ok_or(RemoveNodeError::NodeChainNotFound)?;

        let old_chain_label = chain.label().to_string().into_boxed_str();
        let remove_result = chain.remove_node(node_id, &old_chain_label);
        let new_chain_label = chain.label().to_string().into_boxed_str();
        let chain_node_count = chain.node_count();
        let genesis_hash = *chain.genesis_hash();

        // Is the chain empty? Remove if so and clean up indexes to it
        if chain_node_count == 0 {
            self.chains_by_label.remove(&old_chain_label);
            self.chains_by_genesis_hash.remove(&genesis_hash);
            self.chains.remove(chain_id);
        }

        // Make sure chains always referenced by their most common label:
        if remove_result.chain_renamed {
            self.chains_by_label.remove(&old_chain_label);
            self.chains_by_label.insert(new_chain_label.clone(), chain_id);
        }

        Ok(RemovedNode {
            old_chain_label,
            new_chain_label,
            chain_node_count: chain_node_count,
            has_chain_label_changed: remove_result.chain_renamed
        })
    }

    /// Attempt to update the best block seen, given a node and block.
    /// Returns a boolean which denotes whether the output is for finalization feeds (true) or not (false).
    pub fn update_node(&mut self, node_id: NodeId, payload: Payload, feed: &mut FeedMessageSerializer) -> bool {
        let chain_id = match self.chains_by_node.get(&node_id) {
            Some(chain_id) => *chain_id,
            None => { log::error!("Cannot find chain_id for node with ID {}", node_id); return false }
        };

        let chain = match self.chains.get_mut(chain_id) {
            Some(chain) => chain,
            None => { log::error!("Cannot find chain for node with ID {}", node_id); return false }
        };

        chain.update_node(&mut self.nodes, node_id, payload, feed)
    }

    /// Update the location for a node. Return `false` if the node was not found.
    pub fn update_node_location(&mut self, node_id: NodeId, location: find_location::Location) -> bool {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.update_location(location);
            true
        } else {
            false
        }
    }
}


/// When we ask for a chain, we get this struct back. This ensures that we have
/// a consistent public interface, and don't expose methods on [`Chain`] that
/// aren't really intended for use outside of [`State`] methods. Any modification
/// of a chain needs to go through [`State`].
pub struct StateChain<'a> {
    state: &'a State,
    chain: &'a Chain
}

impl <'a> StateChain<'a> {
    pub fn label(&self) -> &'a str {
        self.chain.label()
    }
    pub fn genesis_hash(&self) -> &'a BlockHash {
        self.chain.genesis_hash()
    }
    pub fn node_count(&self) -> usize {
        self.chain.node_count()
    }
    pub fn best_block(&self) -> &'a Block {
        self.chain.best_block()
    }
    pub fn timestamp(&self) -> Timestamp {
        self.chain.timestamp().unwrap_or(0)
    }
    pub fn average_block_time(&self) -> Option<u64> {
        self.chain.average_block_time()
    }
    pub fn finalized_block(&self) -> &'a Block {
        self.chain.finalized_block()
    }
    pub fn iter_nodes(&self) -> impl Iterator<Item=(NodeId, &'a Node)> + 'a {
        let state = self.state;
        self.chain.node_ids().filter_map(move |id| {
            Some((id, state.nodes.get(id)?))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn node(name: &str, chain: &str) -> NodeDetails {
        NodeDetails {
            chain: chain.into(),
            name: name.into(),
            implementation: "Bar".into(),
            version: "0.1".into(),
            validator: None,
            network_id: None,
            startup_time: None
        }
    }

    #[test]
    fn adding_a_node_returns_expected_response() {
        let mut state = State::new(None);

        let chain1_genesis = BlockHash::from_low_u64_be(1);

        let add_result = state.add_node(
            chain1_genesis,
            node("A", "Chain One")
        );

        let add_node_result = match add_result {
            AddNodeResult::ChainOnDenyList => panic!("Chain not on deny list"),
            AddNodeResult::ChainOverQuota => panic!("Chain not Overquota"),
            AddNodeResult::NodeAddedToChain(details) => details
        };

        assert_eq!(add_node_result.id, 0);
        assert_eq!(&*add_node_result.old_chain_label, "");
        assert_eq!(&*add_node_result.new_chain_label, "Chain One");
        assert_eq!(add_node_result.chain_node_count, 1);
        assert_eq!(add_node_result.has_chain_label_changed, true);

        let add_result = state.add_node(
            chain1_genesis,
            node("A", "Chain One")
        );

        let add_node_result = match add_result {
            AddNodeResult::ChainOnDenyList => panic!("Chain not on deny list"),
            AddNodeResult::ChainOverQuota => panic!("Chain not Overquota"),
            AddNodeResult::NodeAddedToChain(details) => details
        };

        assert_eq!(add_node_result.id, 1);
        assert_eq!(&*add_node_result.old_chain_label, "Chain One");
        assert_eq!(&*add_node_result.new_chain_label, "Chain One");
        assert_eq!(add_node_result.chain_node_count, 2);
        assert_eq!(add_node_result.has_chain_label_changed, false);
    }

    #[test]
    fn adding_and_removing_nodes_updates_chain_label_mapping() {
        let mut state = State::new(None);

        let chain1_genesis = BlockHash::from_low_u64_be(1);
        state.add_node(chain1_genesis, node("A", "Chain One")); // 0

        assert_eq!(state.get_chain_by_node_id(0).expect("Chain should exist").label(), "Chain One");
        assert!(state.get_chain_by_label("Chain One").is_some());
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());

        state.add_node(chain1_genesis, node("B", "Chain Two")); // 1

        // Chain name hasn't changed yet; "Chain One" as common as "Chain Two"..
        assert_eq!(state.get_chain_by_node_id(0).expect("Chain should exist").label(), "Chain One");
        assert!(state.get_chain_by_label("Chain One").is_some());
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());

        state.add_node(chain1_genesis, node("B", "Chain Two")); // 2

        // Chain name has changed; "Chain Two" the winner now..
        assert_eq!(state.get_chain_by_node_id(0).expect("Chain should exist").label(), "Chain Two");
        assert!(state.get_chain_by_label("Chain One").is_none());
        assert!(state.get_chain_by_label("Chain Two").is_some());
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());

        state.remove_node(1).expect("Removal OK (id: 1)");
        state.remove_node(2).expect("Removal OK (id: 2");

        // Removed both "Chain Two" nodes; dominant name now "Chain One" again..
        assert_eq!(state.get_chain_by_node_id(0).expect("Chain should exist").label(), "Chain One");
        assert!(state.get_chain_by_label("Chain One").is_some());
        assert!(state.get_chain_by_label("Chain Two").is_none());
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());
    }

    #[test]
    fn chain_removed_when_last_node_is() {
        let mut state = State::new(None);

        let chain1_genesis = BlockHash::from_low_u64_be(1);
        state.add_node(chain1_genesis, node("A", "Chain One")); // 0

        assert!(state.get_chain_by_label("Chain One").is_some());
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());
        assert_eq!(state.iter_chains().count(), 1);

        state.remove_node(0);

        assert!(state.get_chain_by_label("Chain One").is_none());
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_none());
        assert_eq!(state.iter_chains().count(), 0);
    }
}