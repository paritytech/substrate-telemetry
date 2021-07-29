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
        ip: IpAddr,
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
