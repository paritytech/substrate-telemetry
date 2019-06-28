use serde::Serialize;
use serde_json::to_writer;
use chrono::{DateTime, Utc};
use crate::types::{BlockNumber, NodeId, NodeDetails, NodeStats, NodeHardware, NodeLocation, BlockDetails};

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
        let _  = to_writer(&mut self.buffer, &msg);
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
impl FeedMessage for AddedNode<'_> { const ACTION: u8 = 0x03; }
impl FeedMessage for RemovedNode { const ACTION: u8 = 0x04; }
impl FeedMessage for ImportedBlock<'_> { const ACTION: u8 = 0x06; }
impl FeedMessage for AddedChain<'_> { const ACTION: u8 = 0x0B; }
impl FeedMessage for RemovedChain<'_> { const ACTION: u8 = 0x0C; }
impl FeedMessage for SubscribedTo<'_> { const ACTION: u8 = 0x0D; }
impl FeedMessage for UnsubscribedFrom<'_> { const ACTION: u8 = 0x0E; }
impl FeedMessage for Pong<'_> { const ACTION: u8 = 0x0F; }

#[derive(Serialize)]
pub struct Version(pub usize);
  // BestBlock        : 0x01 as 0x01,
  // BestFinalized    : 0x02 as 0x02,

  // export interface BestBlockMessage extends MessageBase {
  //   action: typeof Actions.BestBlock;
  //   payload: [BlockNumber, Timestamp, Maybe<Milliseconds>];
  // }

  // export interface BestFinalizedBlockMessage extends MessageBase {
  //   action: typeof Actions.BestFinalized;
  //   payload: [BlockNumber, BlockHash];
  // }

  // export interface AddedNodeMessage extends MessageBase {
  //   action: typeof Actions.AddedNode;
  //   payload: [NodeId, NodeDetails, NodeStats, NodeHardware, BlockDetails, Maybe<NodeLocation>];
  // }


  // export interface RemovedNodeMessage extends MessageBase {
  //   action: typeof Actions.RemovedNode;
  //   payload: NodeId;
  // }
#[derive(Serialize)]
pub struct BestBlock(pub BlockNumber, pub DateTime<Utc>, pub Option<u64>);

#[derive(Serialize)]
pub struct AddedNode<'a>(pub NodeId, pub &'a NodeDetails, pub &'a NodeStats, pub NodeHardware<'a>, pub NodeLocation<'a>);

#[derive(Serialize)]
pub struct RemovedNode(pub NodeId);

#[derive(Serialize)]
pub struct ImportedBlock<'a>(pub NodeId, pub &'a BlockDetails);

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
