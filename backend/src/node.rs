use serde::Deserialize;

pub mod message;
pub mod connector;

use message::{NodeMessage, Block};

pub type NodeId = usize;

#[derive(Deserialize, Debug)]
pub struct NodeDetails {
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
}

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Best block
    best: Block,
}

impl Node {
    pub fn new(details: NodeDetails) -> Self {
        Node {
            details,
            best: Block::zero(),
        }
    }

    pub fn name(&self) -> &str {
        &self.details.name
    }

    pub fn update(&mut self, msg: NodeMessage) {
        if let Some(block) = msg.details.best_block() {
            if block.height > self.best.height {
                self.best = *block;
            }
        }
    }
}
