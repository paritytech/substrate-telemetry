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

//! This is the internal representation of telemetry messages sent from nodes.
//! There is a separate JSON representation of these types, because internally we want to be
//! able to serialize these messages to bincode, and various serde attributes aren't compatible
//! with this, hence this separate internal representation.

use crate::node_types::{Block, BlockHash, BlockNumber, NodeDetails};
use serde::{Deserialize, Serialize};

pub type NodeMessageId = u64;

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeMessage {
    V1 { payload: Payload },
    V2 { id: NodeMessageId, payload: Payload },
}

impl NodeMessage {
    /// Returns the ID associated with the node message, or 0
    /// if the message has no ID.
    pub fn id(&self) -> NodeMessageId {
        match self {
            NodeMessage::V1 { .. } => 0,
            NodeMessage::V2 { id, .. } => *id,
        }
    }
    /// Return the payload associated with the message.
    pub fn into_payload(self) -> Payload {
        match self {
            NodeMessage::V1 { payload, .. } | NodeMessage::V2 { payload, .. } => payload,
        }
    }
}

impl From<NodeMessage> for Payload {
    fn from(msg: NodeMessage) -> Payload {
        msg.into_payload()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    SystemConnected(SystemConnected),
    SystemInterval(SystemInterval),
    BlockImport(Block),
    NotifyFinalized(Finalized),
    AfgAuthoritySet(AfgAuthoritySet),
    HwBench(NodeHwBench),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemConnected {
    pub genesis_hash: BlockHash,
    pub node: NodeDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInterval {
    pub peers: Option<u64>,
    pub txcount: Option<u64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<BlockHash>,
    pub block: Option<Block>,
    pub used_state_cache_size: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Finalized {
    pub hash: BlockHash,
    pub height: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeHwBench {
    pub cpu_hashrate_score: u64,
    pub memory_memcpy_score: u64,
    pub disk_sequential_write_score: Option<u64>,
    pub disk_random_write_score: Option<u64>,
}

impl Payload {
    pub fn best_block(&self) -> Option<&Block> {
        match self {
            Payload::BlockImport(block) => Some(block),
            Payload::SystemInterval(SystemInterval { block, .. }) => block.as_ref(),
            _ => None,
        }
    }

    pub fn finalized_block(&self) -> Option<Block> {
        match self {
            Payload::SystemInterval(ref interval) => Some(Block {
                hash: interval.finalized_hash?,
                height: interval.finalized_height?,
            }),
            Payload::NotifyFinalized(ref finalized) => Some(Block {
                hash: finalized.hash,
                height: finalized.height.parse().ok()?,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayString;
    use bincode::Options;

    // Without adding a derive macro and marker trait (and enforcing their use), we don't really
    // know whether things can (de)serialize to bincode or not at runtime without failing unless
    // we test the different types we want to (de)serialize ourselves. We just need to test each
    // type, not each variant.
    fn bincode_can_serialize_and_deserialize<'de, T>(item: T)
    where
        T: Serialize + serde::de::DeserializeOwned,
    {
        let bytes = bincode::serialize(&item).expect("Serialization should work");
        let _: T = bincode::deserialize(&bytes).expect("Deserialization should work");
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_system_connected() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::SystemConnected(SystemConnected {
                genesis_hash: BlockHash::zero(),
                node: NodeDetails {
                    chain: "foo".into(),
                    name: "foo".into(),
                    implementation: "foo".into(),
                    version: "foo".into(),
                    target_arch: Some("x86_64".into()),
                    target_os: Some("linux".into()),
                    target_env: Some("env".into()),
                    validator: None,
                    network_id: ArrayString::new(),
                    startup_time: None,
                    sysinfo: None,
                    ip: Some("127.0.0.1".into()),
                },
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_system_interval() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::SystemInterval(SystemInterval {
                peers: None,
                txcount: None,
                bandwidth_upload: None,
                bandwidth_download: None,
                finalized_height: None,
                finalized_hash: None,
                block: None,
                used_state_cache_size: None,
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_block_import() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::BlockImport(Block {
                hash: BlockHash([0; 32]),
                height: 0,
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_notify_finalized() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::NotifyFinalized(Finalized {
                hash: BlockHash::zero(),
                height: "foo".into(),
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_afg_authority_set() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::AfgAuthoritySet(AfgAuthoritySet {
                authority_id: "foo".into(),
            }),
        });
    }

    #[test]
    fn bincode_block_zero() {
        let raw = Block::zero();

        let bytes = bincode::options().serialize(&raw).unwrap();

        let deserialized: Block = bincode::options().deserialize(&bytes).unwrap();

        assert_eq!(raw.hash, deserialized.hash);
        assert_eq!(raw.height, deserialized.height);
    }
}
