//! Internal messages passed between the shard and telemetry core.

use std::net::IpAddr;

use crate::id_type;
use crate::node_message::Payload;
use crate::node_types::{BlockHash, NodeDetails};
use serde::{Deserialize, Serialize};

id_type! {
    /// The shard-local ID of a given node, where a single connection
    /// might send data on behalf of more than one chain.
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ShardNodeId(usize);
}

/// Message sent from a telemetry shard to the telemetry core
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FromShardAggregator {
    /// Get information about a new node, including it's IP
    /// address and chain genesis hash.
    AddNode {
        ip: Option<IpAddr>,
        node: NodeDetails,
        local_id: ShardNodeId,
        genesis_hash: BlockHash,
    },
    /// A message payload with updated details for a node
    UpdateNode {
        local_id: ShardNodeId,
        payload: Payload,
    },
    /// Inform the telemetry core that a node has been removed
    RemoveNode { local_id: ShardNodeId },
}

/// Message sent form the telemetry core to a telemetry shard
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FromTelemetryCore {
    Mute {
        local_id: ShardNodeId,
        reason: MuteReason,
    },
}

/// Why is the thing being muted?
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MuteReason {
    Overquota,
    ChainNotAllowed,
}
