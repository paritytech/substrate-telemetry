use crate::types::{NodeId, NodeDetails, NodeStats, NodeHardware, NodeLocation, BlockNumber};

pub mod message;
pub mod connector;

use message::{NodeMessage, Details, Block};
use std::time::Instant;

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Basic stats
    stats: NodeStats,
    /// Best block
    best: Block,
    /// Timestamp of best block
    block_timestamp: Instant,
    /// Block time delta
    block_time: u64,
}

impl Node {
    pub fn new(details: NodeDetails) -> Self {
        Node {
            details,
            stats: NodeStats {
                txcount: 0,
                peers: 0,
            },
            best: Block::zero(),
            block_timestamp: Instant::now(),
            block_time: 0
        }
    }

    pub fn details(&self) -> &NodeDetails {
        &self.details
    }

    pub fn stats(&self) -> &NodeStats {
        &self.stats
    }

    pub fn best(&self) -> &Block {
        &self.best
    }

    pub fn hardware(&self) -> NodeHardware {
        (&[], &[], &[], &[], &[])
    }

    pub fn location(&self) -> NodeLocation {
        (0.0, 0.0, "")
    }

    pub fn block_time(&self) -> u64 {
        self.block_time
    }

    pub fn update_block_time(&mut self, block_height: BlockNumber, timestamp: Instant) {
        if block_height > self.best.height {
            self.block_time = (timestamp - self.block_timestamp).as_millis() as u64;
            self.block_timestamp = timestamp; 
        }
    }

    pub fn update(&mut self, msg: NodeMessage) {
        if let Some(block) = msg.details.best_block() {
            if block.height > self.best.height {
                self.best = *block;
            }
        }

        match msg.details {
            Details::SystemInterval(ref interval) => {
                self.stats = interval.stats;
            }
            _ => ()
        }
    }
}
