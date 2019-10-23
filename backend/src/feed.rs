use serde::Serialize;
use serde_json::to_writer;
use chrono::{DateTime, Utc};
use crate::types::{
    BlockNumber, BlockHash, NodeId, NodeDetails, NodeStats, NodeHardware, NodeLocation, BlockDetails
};

pub mod connector;

use connector::Serialized;

pub trait FeedMessage: Serialize {
    const ACTION: u8;
}

pub struct FeedMessageSerializer {
    /// Current buffer,
    buffer: Vec<u8>,
}

impl FeedMessageSerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push<Message>(&mut self, msg: Message)
    where
        Message: FeedMessage,
    {
        let glue = match self.buffer.len() {
            0 => b'[',
            _ => b',',
        };

        self.buffer.push(glue);
        let _ = to_writer(&mut self.buffer, &Message::ACTION);
        self.buffer.push(b',');
        let _ = to_writer(&mut self.buffer, &msg);
    }

    pub fn finalize(&mut self) -> Option<Serialized> {
        if self.buffer.len() == 0 {
            return None;
        }

        self.buffer.push(b']');
        let bytes = self.buffer[..].into();
        self.buffer.clear();

        Some(Serialized(bytes))
    }
}

impl FeedMessage for Version { const ACTION: u8 = 0x00; }
impl FeedMessage for BestBlock { const ACTION: u8 = 0x01; }
impl FeedMessage for BestFinalized { const ACTION: u8 = 0x02; }
impl FeedMessage for AddedNode<'_> { const ACTION: u8 = 0x03; }
impl FeedMessage for RemovedNode { const ACTION: u8 = 0x04; }
impl FeedMessage for LocatedNode<'_> { const ACTION: u8 = 0x05; }
impl FeedMessage for ImportedBlock<'_> { const ACTION: u8 = 0x06; }
impl FeedMessage for FinalizedBlock { const ACTION: u8 = 0x07; }
impl FeedMessage for NodeStatsUpdate<'_> { const ACTION: u8 = 0x08; }
impl FeedMessage for Hardware<'_> { const ACTION: u8 = 0x09; }
impl FeedMessage for TimeSync { const ACTION: u8 = 0x0A; }
impl FeedMessage for AddedChain<'_> { const ACTION: u8 = 0x0B; }
impl FeedMessage for RemovedChain<'_> { const ACTION: u8 = 0x0C; }
impl FeedMessage for SubscribedTo<'_> { const ACTION: u8 = 0x0D; }
impl FeedMessage for UnsubscribedFrom<'_> { const ACTION: u8 = 0x0E; }
impl FeedMessage for Pong<'_> { const ACTION: u8 = 0x0F; }

#[derive(Serialize)]
pub struct Version(pub usize);

#[derive(Serialize)]
pub struct BestBlock(pub BlockNumber, pub DateTime<Utc>, pub Option<u64>);

#[derive(Serialize)]
pub struct BestFinalized(pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct AddedNode<'a>(pub NodeId, pub &'a NodeDetails, pub &'a NodeStats, pub NodeHardware<'a>,
                         pub &'a BlockDetails, pub Option<&'a NodeLocation>);

#[derive(Serialize)]
pub struct RemovedNode(pub NodeId);

#[derive(Serialize)]
pub struct LocatedNode<'a>(pub NodeId, pub f32, pub f32, pub &'a str);

#[derive(Serialize)]
pub struct ImportedBlock<'a>(pub NodeId, pub &'a BlockDetails);

#[derive(Serialize)]
pub struct FinalizedBlock(pub NodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct NodeStatsUpdate<'a>(pub NodeId, pub &'a NodeStats);

#[derive(Serialize)]
pub struct Hardware<'a>(pub NodeId, pub NodeHardware<'a>);

#[derive(Serialize)]
pub struct TimeSync(pub u64);

#[derive(Serialize)]
pub struct AddedChain<'a>(pub &'a str, pub usize);

#[derive(Serialize)]
pub struct RemovedChain<'a>(pub &'a str);

#[derive(Serialize)]
pub struct SubscribedTo<'a>(pub &'a str);

#[derive(Serialize)]
pub struct UnsubscribedFrom<'a>(pub &'a str);

#[derive(Serialize)]
pub struct Pong<'a>(pub &'a str);
