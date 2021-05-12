use std::net::Ipv4Addr;

use crate::node::Payload;
use serde::{Deserialize, Serialize};

/// Alias for the ID of the node connection
type ShardConnId = u32;

#[derive(Deserialize, Serialize)]
pub enum ShardMessage {
    /// Get a connection id for a new node, passing IPv4
    New(Ipv4Addr),
    /// Send a message payload for a given node
    Payload {
        conn_id: ShardConnId,
        payload: Payload,
    },
}
