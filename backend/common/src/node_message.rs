//! This is the internal represenation of telemetry messages sent from nodes.
//! There is a separate JSON representation of these types, because internally we want to be
//! able to serialize these messages to bincode, and various serde attribtues aren't compatible
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
    TxPoolImport,
    AfgFinalized(AfgFinalized),
    AfgReceivedPrecommit(AfgReceived),
    AfgReceivedPrevote(AfgReceived),
    AfgReceivedCommit(AfgReceived),
    AfgAuthoritySet(AfgAuthoritySet),
    AfgFinalizedBlocksUpTo,
    AuraPreSealedBlock,
    PreparedBlockForProposing,
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
pub struct AfgFinalized {
    pub finalized_hash: BlockHash,
    pub finalized_number: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AfgReceived {
    pub target_hash: BlockHash,
    pub target_number: Box<str>,
    pub voter: Option<Box<str>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
    pub authorities: Box<str>,
    pub authority_set_id: Box<str>,
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
                    validator: None,
                    network_id: None,
                    startup_time: None,
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
    fn bincode_can_serialize_and_deserialize_node_message_tx_pool_import() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::TxPoolImport,
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_afg_finalized() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::AfgFinalized(AfgFinalized {
                finalized_hash: BlockHash::zero(),
                finalized_number: "foo".into(),
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_afg_received() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::AfgReceivedPrecommit(AfgReceived {
                target_hash: BlockHash::zero(),
                target_number: "foo".into(),
                voter: None,
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_afg_authority_set() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::AfgAuthoritySet(AfgAuthoritySet {
                authority_id: "foo".into(),
                authorities: "foo".into(),
                authority_set_id: "foo".into(),
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
