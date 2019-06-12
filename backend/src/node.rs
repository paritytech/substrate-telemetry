use crate::types::{NodeId, NodeDetails, NodeStats, NodeHardware};

pub mod message;
pub mod connector;

use message::{NodeMessage, Details, Block};

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Basic stats
    stats: NodeStats,
    /// Best block
    best: Block,
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
        }
    }

    pub fn details(&self) -> &NodeDetails {
        &self.details
    }

    pub fn stats(&self) -> &NodeStats {
        &self.stats
    }

    pub fn hardware(&self) -> NodeHardware {
        (&[], &[], &[], &[], &[])
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
