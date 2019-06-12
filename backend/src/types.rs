use serde::ser::{Serialize, Serializer, SerializeTuple};
use serde::Deserialize;

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

pub type NodeHardware<'a> = (&'a [usize], &'a [usize], &'a [usize], &'a [usize], &'a [usize]);

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
