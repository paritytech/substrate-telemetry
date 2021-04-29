use crate::node::message::Payload;
use serde::Deserialize;

pub mod connector;

/// Alias for the ID of the node connection
type ShardConnId = usize;

#[derive(Deserialize)]
pub struct ShardMessage {
    pub conn_id: ShardConnId,
    pub payload: Payload,
}
