use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use crate::node::NodeDetails;

pub use primitive_types::H256 as BlockHash;
pub type BlockNumber = u64;

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
    pub txcount: u64,
    pub peers: u64,
    pub memory: Option<f64>,
    pub cpu: Option<f64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<BlockHash>,
    #[serde(flatten)]
    pub block: Block,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Block {
    #[serde(rename = "best")]
    pub hash: BlockHash,
    pub height: BlockNumber,
}
