use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::de::IgnoredAny;
use crate::node::{NodeDetails, NodeStats};
use crate::types::{Block, BlockNumber, BlockHash};

#[derive(Deserialize, Debug, Message)]
pub struct NodeMessage {
    pub level: Level,
    pub ts: DateTime<Utc>,
    #[serde(flatten)]
    pub details: Details,
}

#[derive(Deserialize, Debug)]
pub enum Level {
    #[serde(rename = "INFO")]
    Info,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "msg")]
pub enum Details {
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
    pub chain: Box<str>,
    #[serde(flatten)]
    pub node: NodeDetails,
}

#[derive(Deserialize, Debug)]
pub struct SystemInterval {
    #[serde(flatten)]
    pub stats: NodeStats,
    pub memory: Option<f32>,
    pub cpu: Option<f32>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<BlockHash>,
    #[serde(flatten)]
    pub block: Block,
    pub network_state: Option<IgnoredAny>,
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

impl Details {
    pub fn best_block(&self) -> Option<&Block> {
        match self {
            Details::BlockImport(block) | Details::SystemInterval(SystemInterval { block, .. }) => {
                Some(block)
            }
            _ => None,
        }
    }

    pub fn finalized_block(&self) -> Option<Block> {
        match self {
            Details::SystemInterval(ref interval) => {
                Some(Block {
                    hash: interval.finalized_hash?,
                    height: interval.finalized_height?,
                })
            },
            Details::NotifyFinalized(ref finalized) => {
                Some(Block {
                    hash: finalized.hash,
                    height: finalized.height.parse().ok()?
                })
            },
            _ => None
        }
    }
}
