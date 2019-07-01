use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use crate::node::{NodeDetails, NodeStats};
use crate::types::{BlockNumber, BlockHash};

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
    #[serde(rename = "block.import")]
    BlockImport(Block),
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
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Block {
    #[serde(rename = "best")]
    pub hash: BlockHash,
    pub height: BlockNumber,
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
}
