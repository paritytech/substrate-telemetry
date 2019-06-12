use serde::ser::{Serialize, Serializer, SerializeTuple};
use serde::Deserialize;

pub mod message;
pub mod connector;

use message::{NodeMessage, Details, Block};

pub type NodeId = usize;

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

#[derive(Deserialize, Debug)]
pub struct NodeDetails {
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct NodeStats {
    pub txcount: u64,
    pub peers: u64,
}

impl Serialize for NodeDetails {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(6)?;
        tup.serialize_element(&self.name)?;
        tup.serialize_element(&self.implementation)?;
        tup.serialize_element(&self.version)?;
        tup.serialize_element::<Option<String>>(&None)?; // Maybe<Address>
        tup.serialize_element::<Option<usize>>(&None)?; // Maybe<NetworkId>
        tup.serialize_element(""); // Address
        tup.end()
    }
}

impl Serialize for NodeStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.txcount)?;
        tup.serialize_element(&self.peers)?;
        tup.end()
    }
}
