use std::net::IpAddr;

use crate::node::Payload;
use crate::types::{NodeDetails, BlockHash};
use crate::assign_id::Id;
use serde::{Deserialize, Serialize};

/// The shard-local ID of a given node, where a single connection
/// might send data on behalf of more than one chain.
pub type LocalId = Id;

/// A global ID assigned to messages from each different pair of ConnId+LocalId.
pub type GlobalId = usize;

/// Message sent from the shard to the backend core
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FromShardAggregator {
    /// Get information about a new node, passing IPv4
    AddNode {
    	ip: Option<IpAddr>,
    	node: NodeDetails,
    	local_id: LocalId,
        genesis_hash: BlockHash
    },
    /// Send a message payload to update details for a node
    UpdateNode {
        local_id: LocalId,
        payload: Payload,
    },
    /// Inform the core that a node has been removed
    RemoveNode {
        local_id: LocalId
    }
}

/// Message sent form the backend core to the shard
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FromTelemetryCore {
	Mute {
		local_id: LocalId
	}
}
