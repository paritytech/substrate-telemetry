// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::node::Node;
use crate::feed_message::{ChainStats, FeedMessageSerializer};
use crate::find_location;
use common::node_message::Payload;
use common::node_types::{AppPeriod, Block, BlockHash, NodeDetails, Timestamp, VerifierBlockInfos};
use common::{id_type, DenseMap};
use std::collections::{HashMap, HashSet};
use std::iter::IntoIterator;

use super::chain::{self, Chain, ChainNodeId};

id_type! {
    /// A globally unique Chain ID.
    pub struct ChainId(usize)
}

/// A "global" Node ID is a composite of the ID of the chain it's
/// on, and it's chain local ID.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(ChainId, ChainNodeId);

impl NodeId {
    pub fn get_chain_node_id(&self) -> ChainNodeId {
        self.1
    }
}

/// Our state contains node and chain information
pub struct State {
    chains: DenseMap<ChainId, Chain>,

    /// Find the right chain given various details.
    chains_by_genesis_hash: HashMap<BlockHash, ChainId>,

    /// Chain labels that we do not want to allow connecting.
    denylist: HashSet<String>,

    /// How many nodes from third party chains are allowed to connect
    /// before we prevent connections from them.
    max_third_party_nodes: usize,
}

/// Adding a node to a chain leads to this result.
pub enum AddNodeResult<'a> {
    /// The chain is on the "deny list", so we can't add the node
    ChainOnDenyList,
    /// The chain is over quota (too many nodes connected), so can't add the node
    ChainOverQuota,
    /// The node was added to the chain
    NodeAddedToChain(NodeAddedToChain<'a>),
}

#[cfg(test)]
impl<'a> AddNodeResult<'a> {
    pub fn unwrap_id(&self) -> NodeId {
        match &self {
            AddNodeResult::NodeAddedToChain(d) => d.id,
            _ => panic!("Attempt to unwrap_id on AddNodeResult that did not succeed"),
        }
    }
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
    pub has_chain_label_changed: bool,
}

/// if removing a node is successful, we get this information back.
pub struct RemovedNode {
    /// How many nodes remain on the chain (0 if the chain was removed)
    pub chain_node_count: usize,
    /// Has the chain label been updated?
    pub has_chain_label_changed: bool,
    /// The old label of the chain.
    pub old_chain_label: Box<str>,
    /// Genesis hash of the chain to be updated.
    pub chain_genesis_hash: BlockHash,
    /// The new label of the chain.
    pub new_chain_label: Box<str>,
}

impl State {
    pub fn new<T: IntoIterator<Item = String>>(denylist: T, max_third_party_nodes: usize) -> State {
        State {
            chains: DenseMap::new(),
            chains_by_genesis_hash: HashMap::new(),
            denylist: denylist.into_iter().collect(),
            max_third_party_nodes,
        }
    }

    pub fn iter_chains(&self) -> impl Iterator<Item = StateChain<'_>> {
        self.chains
            .iter()
            .map(move |(_, chain)| StateChain { chain })
    }

    pub fn get_chain_by_node_id(&self, node_id: NodeId) -> Option<StateChain<'_>> {
        self.chains.get(node_id.0).map(|chain| StateChain { chain })
    }

    pub fn get_chain_by_genesis_hash(&self, genesis_hash: &BlockHash) -> Option<StateChain<'_>> {
        self.chains_by_genesis_hash
            .get(genesis_hash)
            .and_then(|&chain_id| self.chains.get(chain_id))
            .map(|chain| StateChain { chain })
    }

    pub fn add_node(
        &mut self,
        genesis_hash: BlockHash,
        node_details: NodeDetails,
    ) -> AddNodeResult<'_> {
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
                let max_nodes = match chain::is_first_party_network(&genesis_hash) {
                    true => usize::MAX,
                    false => self.max_third_party_nodes,
                };
                let chain_id = self.chains.add(Chain::new(genesis_hash, max_nodes));
                self.chains_by_genesis_hash.insert(genesis_hash, chain_id);
                chain_id
            }
        };

        // Get the chain.
        let chain = self.chains.get_mut(chain_id).expect(
            "should be known to exist after the above (unless chains_by_genesis_hash out of sync)",
        );

        let node = Node::new(node_details);
        let old_chain_label = chain.label().into();

        match chain.add_node(node) {
            chain::AddNodeResult::Overquota => AddNodeResult::ChainOverQuota,
            chain::AddNodeResult::Added { id, chain_renamed } => {
                let chain = &*chain;

                AddNodeResult::NodeAddedToChain(NodeAddedToChain {
                    id: NodeId(chain_id, id),
                    node: chain.get_node(id).expect("node added above"),
                    old_chain_label,
                    new_chain_label: chain.label(),
                    chain_node_count: chain.node_count(),
                    has_chain_label_changed: chain_renamed,
                })
            }
        }
    }

    /// Remove a node
    pub fn remove_node(&mut self, NodeId(chain_id, chain_node_id): NodeId) -> Option<RemovedNode> {
        let chain = self.chains.get_mut(chain_id)?;
        let old_chain_label = chain.label().into();

        // Actually remove the node
        let remove_result = chain.remove_node(chain_node_id);

        // Get updated chain details.
        let new_chain_label: Box<str> = chain.label().into();
        let chain_node_count = chain.node_count();
        let chain_genesis_hash = chain.genesis_hash();

        // Is the chain empty? Remove if so and clean up indexes to it
        if chain_node_count == 0 {
            let genesis_hash = chain.genesis_hash();
            self.chains_by_genesis_hash.remove(&genesis_hash);
            self.chains.remove(chain_id);
        }

        Some(RemovedNode {
            old_chain_label,
            new_chain_label,
            chain_node_count,
            chain_genesis_hash,
            has_chain_label_changed: remove_result.chain_renamed,
        })
    }

    /// Attempt to update the best block seen, given a node and block.
    pub fn update_node(
        &mut self,
        NodeId(chain_id, chain_node_id): NodeId,
        payload: Payload,
        feed: &mut FeedMessageSerializer,
    ) {
        let chain = match self.chains.get_mut(chain_id) {
            Some(chain) => chain,
            None => {
                log::error!("Cannot find chain for node with ID {:?}", chain_id);
                return;
            }
        };

        chain.update_node(chain_node_id, payload, feed)
    }

    /// Update the location for a node. Return `false` if the node was not found.
    pub fn update_node_location(
        &mut self,
        NodeId(chain_id, chain_node_id): NodeId,
        location: find_location::Location,
    ) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_id) {
            chain.update_node_location(chain_node_id, location)
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
    chain: &'a Chain,
}

impl<'a> StateChain<'a> {
    pub fn label(&self) -> &'a str {
        self.chain.label()
    }
    pub fn genesis_hash(&self) -> BlockHash {
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
    pub fn nodes_slice(&self) -> &[Option<Node>] {
        self.chain.nodes_slice()
    }
    pub fn stats(&self) -> &ChainStats {
        self.chain.stats()
    }

    pub fn submitted_block(&self) -> &VerifierBlockInfos {
        &self.chain.submitted_block
    }
    pub fn challenged_block(&self) -> &VerifierBlockInfos {
        &self.chain.challenged_block
    }
    pub fn submission_period(&self) -> AppPeriod {
        self.chain.submission_period
    }
    pub fn challenge_period(&self) -> AppPeriod {
        self.chain.challenge_period
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common::node_types::NetworkId;

    fn node(name: &str, chain: &str) -> NodeDetails {
        NodeDetails {
            chain: chain.into(),
            name: name.into(),
            implementation: "Bar".into(),
            target_arch: Some("x86_64".into()),
            target_os: Some("linux".into()),
            target_env: Some("env".into()),
            version: "0.1".into(),
            validator: None,
            network_id: NetworkId::new(),
            startup_time: None,
            sysinfo: None,
            ip: None,
        }
    }

    #[test]
    fn adding_a_node_returns_expected_response() {
        let mut state = State::new(None, 1000);

        let chain1_genesis = BlockHash::from_low_u64_be(1);

        let add_result = state.add_node(chain1_genesis, node("A", "Chain One"));

        let add_node_result = match add_result {
            AddNodeResult::ChainOnDenyList => panic!("Chain not on deny list"),
            AddNodeResult::ChainOverQuota => panic!("Chain not Overquota"),
            AddNodeResult::NodeAddedToChain(details) => details,
        };

        assert_eq!(add_node_result.id, NodeId(0.into(), 0.into()));
        assert_eq!(&*add_node_result.old_chain_label, "");
        assert_eq!(&*add_node_result.new_chain_label, "Chain One");
        assert_eq!(add_node_result.chain_node_count, 1);
        assert_eq!(add_node_result.has_chain_label_changed, true);

        let add_result = state.add_node(chain1_genesis, node("A", "Chain One"));

        let add_node_result = match add_result {
            AddNodeResult::ChainOnDenyList => panic!("Chain not on deny list"),
            AddNodeResult::ChainOverQuota => panic!("Chain not Overquota"),
            AddNodeResult::NodeAddedToChain(details) => details,
        };

        assert_eq!(add_node_result.id, NodeId(0.into(), 1.into()));
        assert_eq!(&*add_node_result.old_chain_label, "Chain One");
        assert_eq!(&*add_node_result.new_chain_label, "Chain One");
        assert_eq!(add_node_result.chain_node_count, 2);
        assert_eq!(add_node_result.has_chain_label_changed, false);
    }

    #[test]
    fn adding_and_removing_nodes_updates_chain_label_mapping() {
        let mut state = State::new(None, 1000);

        let chain1_genesis = BlockHash::from_low_u64_be(1);
        let node_id0 = state
            .add_node(chain1_genesis, node("A", "Chain One")) // 0
            .unwrap_id();

        assert_eq!(
            state
                .get_chain_by_node_id(node_id0)
                .expect("Chain should exist")
                .label(),
            "Chain One"
        );
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());

        let node_id1 = state
            .add_node(chain1_genesis, node("B", "Chain Two")) // 1
            .unwrap_id();

        // Chain name hasn't changed yet; "Chain One" as common as "Chain Two"..
        assert_eq!(
            state
                .get_chain_by_node_id(node_id0)
                .expect("Chain should exist")
                .label(),
            "Chain One"
        );
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());

        let node_id2 = state
            .add_node(chain1_genesis, node("B", "Chain Two"))
            .unwrap_id(); // 2

        // Chain name has changed; "Chain Two" the winner now..
        assert_eq!(
            state
                .get_chain_by_node_id(node_id0)
                .expect("Chain should exist")
                .label(),
            "Chain Two"
        );
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());

        state.remove_node(node_id1).expect("Removal OK (id: 1)");
        state.remove_node(node_id2).expect("Removal OK (id: 2)");

        // Removed both "Chain Two" nodes; dominant name now "Chain One" again..
        assert_eq!(
            state
                .get_chain_by_node_id(node_id0)
                .expect("Chain should exist")
                .label(),
            "Chain One"
        );
        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());
    }

    #[test]
    fn chain_removed_when_last_node_is() {
        let mut state = State::new(None, 1000);

        let chain1_genesis = BlockHash::from_low_u64_be(1);
        let node_id = state
            .add_node(chain1_genesis, node("A", "Chain One")) // 0
            .unwrap_id();

        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_some());
        assert_eq!(state.iter_chains().count(), 1);

        state.remove_node(node_id);

        assert!(state.get_chain_by_genesis_hash(&chain1_genesis).is_none());
        assert_eq!(state.iter_chains().count(), 0);
    }
}
