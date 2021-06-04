use crate::types::{Block, BlockHash, BlockNumber, ConnId, NodeDetails};
use crate::util::{Hash, NullAny};
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde::ser::Serializer;

#[derive(Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(untagged)]
pub enum NodeMessage {
    V1 {
        #[serde(flatten)]
        payload: Payload,
    },
    V2 {
        id: ConnId,
        payload: Payload,
    },
}

impl NodeMessage {
    /// Returns the connection ID or 0 if there is no ID.
    pub fn id(&self) -> ConnId {
        match self {
            NodeMessage::V1 { .. } => 0,
            NodeMessage::V2 { id, .. } => *id,
        }
    }
}

impl From<NodeMessage> for Payload {
    fn from(msg: NodeMessage) -> Payload {
        match msg {
            NodeMessage::V1 { payload, .. } | NodeMessage::V2 { payload, .. } => payload,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "msg")]
pub enum Payload {
    #[serde(rename = "system.connected")]
    SystemConnected(SystemConnected),
    #[serde(rename = "system.interval")]
    SystemInterval(SystemInterval),
    #[serde(rename = "block.import")]
    BlockImport(Block),
    #[serde(rename = "notify.finalized")]
    NotifyFinalized(Finalized),
    #[serde(rename = "txpool.import")]
    TxPoolImport(NullAny),
    // #[serde(rename = "afg.finalized")]
    // AfgFinalized(AfgFinalized),
    // #[serde(rename = "afg.received_precommit")]
    // AfgReceivedPrecommit(AfgReceivedPrecommit),
    // #[serde(rename = "afg.received_prevote")]
    // AfgReceivedPrevote(AfgReceivedPrevote),
    // #[serde(rename = "afg.received_commit")]
    // AfgReceivedCommit(AfgReceivedCommit),
    // #[serde(rename = "afg.authority_set")]
    // AfgAuthoritySet(AfgAuthoritySet),
    // #[serde(rename = "afg.finalized_blocks_up_to")]
    // AfgFinalizedBlocksUpTo(NullAny),
    // #[serde(rename = "aura.pre_sealed_block")]
    // AuraPreSealedBlock(NullAny),
    #[serde(rename = "prepared_block_for_proposing")]
    PreparedBlockForProposing(NullAny),
}

impl Serialize for Payload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use Payload::*;

        match self {
            SystemConnected(val) => serializer.serialize_newtype_variant("Payload", 0, "system.connected", val),
            SystemInterval(val) => serializer.serialize_newtype_variant("Payload", 1, "system.interval", val),
            BlockImport(val) => serializer.serialize_newtype_variant("Payload", 3, "block.import", val),
            NotifyFinalized(val) => serializer.serialize_newtype_variant("Payload", 4, "notify.finalized", val),
            TxPoolImport(_) => serializer.serialize_unit_variant("Payload", 3, "txpool.import"),
            PreparedBlockForProposing(_) => serializer.serialize_unit_variant("Payload", 4, "prepared_block_for_proposing"),
            _ => unimplemented!()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SystemConnected {
    pub genesis_hash: Hash,
    #[serde(flatten)]
    pub node: NodeDetails,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SystemInterval {
    pub peers: Option<u64>,
    pub txcount: Option<u64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<BlockHash>,
    #[serde(flatten)]
    pub block: Option<Block>,
    pub used_state_cache_size: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Finalized {
    #[serde(rename = "best")]
    pub hash: BlockHash,
    pub height: Box<str>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
    pub authorities: Box<str>,
    pub authority_set_id: Box<str>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AfgFinalized {
    pub finalized_hash: BlockHash,
    pub finalized_number: Box<str>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AfgReceived {
    pub target_hash: BlockHash,
    pub target_number: Box<str>,
    pub voter: Option<Box<str>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AfgReceivedPrecommit {
    #[serde(flatten)]
    pub received: AfgReceived,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AfgReceivedPrevote {
    #[serde(flatten)]
    pub received: AfgReceived,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AfgReceivedCommit {
    #[serde(flatten)]
    pub received: AfgReceived,
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

    #[test]
    fn message_v1() {
        let json = r#"{"msg":"notify.finalized","level":"INFO","ts":"2021-01-13T12:38:25.410794650+01:00","best":"0x031c3521ca2f9c673812d692fc330b9a18e18a2781e3f9976992f861fd3ea0cb","height":"50"}"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V1 { .. },
            ),
            "message did not match variant V1",
        );
    }

    #[test]
    fn message_v2() {
        let json = r#"{"id":1,"ts":"2021-01-13T12:22:20.053527101+01:00","payload":{"best":"0xcc41708573f2acaded9dd75e07dac2d4163d136ca35b3061c558d7a35a09dd8d","height":"209","msg":"notify.finalized"}}"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V2 { .. },
            ),
            "message did not match variant V2",
        );
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
