use actix::prelude::*;
use serde::Deserialize;
use serde::de::IgnoredAny;
use crate::node::NodeDetails;
use crate::types::{Block, BlockNumber, BlockHash, ConnId};

#[derive(Deserialize, Debug, Message)]
#[rtype(result = "()")]
pub enum NodeMessage {
    V1 {
        #[serde(flatten)]
        payload: Payload,
    },
    V2 {
        id: ConnId,
        #[serde(rename = "payload")]
        payload: Payload,
    },
}

impl NodeMessage {
    /// Returns a reference to the payload.
    pub fn payload(&self) -> &Payload {
        match self {
            NodeMessage::V1 { payload, .. } | NodeMessage::V2 { payload, .. } => payload,
        }
    }

    /// Returns the connection ID or 0 if there is no ID.
    pub fn id(&self) -> ConnId {
        match self {
            NodeMessage::V1 { .. } => 0,
            NodeMessage::V2 { id, .. } => *id,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "msg")]
pub enum Payload {
    #[serde(rename = "node.start")]
    NodeStart(Block),
    #[serde(rename = "system.connected")]
    SystemConnected(SystemConnected),
    #[serde(rename = "system.interval")]
    SystemInterval(SystemInterval),
    #[serde(rename = "system.network_state")]
    SystemNetworkState(IgnoredAny),
    #[serde(rename = "block.import")]
    BlockImport(Block),
    #[serde(rename = "notify.finalized")]
    NotifyFinalized(Finalized),
    #[serde(rename = "txpool.import")]
    TxPoolImport(IgnoredAny),
    #[serde(rename = "afg.finalized")]
    AfgFinalized(AfgFinalized),
    #[serde(rename = "afg.received_precommit")]
    AfgReceivedPrecommit(AfgReceivedPrecommit),
    #[serde(rename = "afg.received_prevote")]
    AfgReceivedPrevote(AfgReceivedPrevote),
    #[serde(rename = "afg.received_commit")]
    AfgReceivedCommit(AfgReceivedCommit),
    #[serde(rename = "afg.authority_set")]
    AfgAuthoritySet(AfgAuthoritySet),
    #[serde(rename = "afg.finalized_blocks_up_to")]
    AfgFinalizedBlocksUpTo(IgnoredAny),
    #[serde(rename = "aura.pre_sealed_block")]
    AuraPreSealedBlock(IgnoredAny),
    #[serde(rename = "prepared_block_for_proposing")]
    PreparedBlockForProposing(IgnoredAny),
}

#[derive(Deserialize, Debug)]
pub struct SystemConnected {
    pub network_id: Option<Box<str>>,
    #[serde(flatten)]
    pub node: NodeDetails,
}

#[derive(Deserialize, Debug)]
pub struct SystemInterval {
    pub peers: Option<u64>,
    pub txcount: Option<u64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<BlockHash>,
    #[serde(flatten)]
    pub block: Option<Block>,
    pub network_state: Option<IgnoredAny>,
    pub used_state_cache_size: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct Finalized {
    #[serde(rename = "best")]
    pub hash: BlockHash,
    pub height: Box<str>,
}

#[derive(Deserialize, Debug)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
    pub authorities: Box<str>,
    pub authority_set_id: Box<str>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgFinalized {
    pub finalized_hash: BlockHash,
    pub finalized_number: Box<str>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgReceived {
    pub target_hash: BlockHash,
    pub target_number: Box<str>,
    pub voter: Option<Box<str>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgReceivedPrecommit {
    #[serde(flatten)]
    pub received: AfgReceived,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgReceivedPrevote {
    #[serde(flatten)]
    pub received: AfgReceived,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgReceivedCommit {
    #[serde(flatten)]
    pub received: AfgReceived,
}

impl Block {
    pub fn zero() -> Self {
        Block {
            hash: BlockHash::from([0; 32]),
            height: 0,
        }
    }
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
            Payload::SystemInterval(ref interval) => {
                Some(Block {
                    hash: interval.finalized_hash?,
                    height: interval.finalized_height?,
                })
            },
            Payload::NotifyFinalized(ref finalized) => {
                Some(Block {
                    hash: finalized.hash,
                    height: finalized.height.parse().ok()?
                })
            },
            _ => None
        }
    }
}
