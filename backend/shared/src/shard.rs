use std::net::Ipv4Addr;

use crate::ws::MuteReason;
use crate::node::Payload;
use crate::types::{NodeId, NodeDetails};
use serde::{Deserialize, Serialize};

/// Alias for the ID of the node connection
pub type ShardConnId = u32;

/// Message sent from the shard to the backend core
#[derive(Deserialize, Serialize, Debug)]
pub enum ShardMessage {
    /// Get a connection id for a new node, passing IPv4
    AddNode {
    	ip: Option<Ipv4Addr>,
    	node: NodeDetails,
    	sid: ShardConnId,
    },
    /// Send a message payload for a given node
    UpdateNode {
        nid: NodeId,
        payload: Payload,
    },
}

/// Message sent form the backend core to the shard
#[derive(Deserialize, Serialize, Debug)]
pub enum BackendMessage {
	Initialize {
		sid: ShardConnId,
		nid: NodeId,
	},
	Mute {
		sid: ShardConnId,
		reason: MuteReason,
	},
}
