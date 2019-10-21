use serde::ser::{Serialize, Serializer, SerializeTuple};
use serde::Deserialize;
use std::sync::Arc;

pub type NodeId = usize;
pub type BlockNumber = u64;
pub use primitive_types::H256 as BlockHash;

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

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct BlockDetails {
    pub block_number: BlockNumber,
    pub block_hash: BlockHash,
    pub block_time: u64,
    pub timestamp: u64,
    pub propagation_time: u64,
}

pub type NodeHardware<'a> = (&'a [f32], &'a [f32], &'a [f64], &'a [f64], &'a [f64]);

#[derive(Debug, Clone)]
pub struct NodeLocation {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Arc<str>,
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
        tup.serialize_element("")?; // Address
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

impl Serialize for BlockDetails {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(5)?;
        tup.serialize_element(&self.block_number)?;
        tup.serialize_element(&self.block_hash)?;
        tup.serialize_element(&self.block_time)?;
        tup.serialize_element(&self.timestamp)?;
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
