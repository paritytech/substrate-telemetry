use serde::ser::{Serialize, Serializer, SerializeTuple};
use serde::Deserialize;

use crate::util::MeanList;

pub type NodeId = usize;
pub type BlockNumber = u64;
pub type Timestamp = u64;
pub type Address = Box<str>;
pub use primitive_types::H256 as BlockHash;

#[derive(Deserialize, Debug)]
pub struct NodeDetails {
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
    pub validator: Option<Box<str>>,
    pub network_id: Option<Box<str>>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeStats {
    pub peers: u64,
    pub txcount: u64,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Block {
    #[serde(rename = "best")]
    pub hash: BlockHash,
    pub height: BlockNumber,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockDetails {
    pub block: Block,
    pub block_time: u64,
    pub block_timestamp: u64,
    pub propagation_time: Option<u64>,
}

#[derive(Default)]
pub struct NodeHardware {
    /// CPU use means
    pub cpu: MeanList<f32>,
    /// Memory use means
    pub memory: MeanList<f32>,
    /// Upload uses means
    pub upload: MeanList<f64>,
    /// Download uses means
    pub download: MeanList<f64>,
    /// Stampchange uses means
    pub chart_stamps: MeanList<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeLocation {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Box<str>,
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
        tup.serialize_element(&self.validator)?;
        tup.serialize_element(&self.network_id)?;
        tup.end()
    }
}

impl Serialize for NodeStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.peers)?;
        tup.serialize_element(&self.txcount)?;
        tup.end()
    }
}

impl Serialize for BlockDetails {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(5)?;
        tup.serialize_element(&self.block.height)?;
        tup.serialize_element(&self.block.hash)?;
        tup.serialize_element(&self.block_time)?;
        tup.serialize_element(&self.block_timestamp)?;
        tup.serialize_element(&self.propagation_time)?;
        tup.end()
    }
}

impl Serialize for NodeLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.latitude)?;
        tup.serialize_element(&self.longitude)?;
        tup.serialize_element(&&*self.city)?;
        tup.end()
    }
}

impl Serialize for NodeHardware {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(5)?;
        tup.serialize_element(self.memory.slice())?;
        tup.serialize_element(self.cpu.slice())?;
        tup.serialize_element(self.upload.slice())?;
        tup.serialize_element(self.download.slice())?;
        tup.serialize_element(self.chart_stamps.slice())?;
        tup.end()
    }
}
