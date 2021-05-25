use std::net::Ipv4Addr;

use crate::node::Payload;
use crate::types::{NodeId, NodeDetails};
use serde::{Deserialize, Serialize};

/// Alias for the ID of the node connection
pub type ShardConnId = u32;

/// Message sent from the shard to backend core
#[derive(Deserialize, Serialize, Debug)]
pub enum ShardMessage {
    /// Get a connection id for a new node, passing IPv4
    AddNode {
    	ip: Option<Ipv4Addr>,
    	node: NodeDetails,
    	sid: ShardConnId,
    },
    /// Send a message payload for a given node
    Payload {
        nid: NodeId,
        payload: Payload,
    },
}

#[derive(Deserialize, Serialize)]
pub struct NodeAdded {
	pub sid: ShardConnId,
	pub nid: NodeId,
}
