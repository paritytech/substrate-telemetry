// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! These types are partly used in [`crate::node_message`], but also stored and used
//! more generally through the application.

use arrayvec::ArrayString;
use serde::ser::{SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};

use crate::{time, MeanList};

pub type BlockNumber = u64;
pub type Timestamp = u64;
pub use primitive_types::H256 as BlockHash;
pub type NetworkId = ArrayString<64>;

pub type AppPeriod = u32;
pub type DigestHash = primitive_types::H256;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierBlockInfos {
    pub digest: DigestHash,
    pub block_number: BlockNumber,
    pub block_hash: BlockHash,
}

impl Default for VerifierBlockInfos {
    fn default() -> Self {
        Self {
            digest: DigestHash::default(),
            block_number: 0,
            block_hash: BlockHash::default(),
        }
    }
}

/// Basic node details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeDetails {
    pub chain: Box<str>,
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
    pub validator: Option<Box<str>>,
    pub network_id: NetworkId,
    pub startup_time: Option<Box<str>>,
    pub target_os: Option<Box<str>>,
    pub target_arch: Option<Box<str>>,
    pub target_env: Option<Box<str>>,
    pub sysinfo: Option<NodeSysInfo>,
    pub ip: Option<Box<str>>,
}

/// Hardware and software information for the node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeSysInfo {
    /// The exact CPU model.
    pub cpu: Option<Box<str>>,
    /// The total amount of memory, in bytes.
    pub memory: Option<u64>,
    /// The number of physical CPU cores.
    pub core_count: Option<u32>,
    /// The Linux kernel version.
    pub linux_kernel: Option<Box<str>>,
    /// The exact Linux distribution used.
    pub linux_distro: Option<Box<str>>,
    /// Whether the node's running under a virtual machine.
    pub is_virtual_machine: Option<bool>,
}

/// Hardware benchmark results for the node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeHwBench {
    /// The CPU speed, as measured in how many MB/s it can hash using the BLAKE2b-256 hash.
    pub cpu_hashrate_score: u64,
    /// Memory bandwidth in MB/s, calculated by measuring the throughput of `memcpy`.
    pub memory_memcpy_score: u64,
    /// Sequential disk write speed in MB/s.
    pub disk_sequential_write_score: Option<u64>,
    /// Random disk write speed in MB/s.
    pub disk_random_write_score: Option<u64>,
}

/// The Details info for a alt-verifier node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierNodeDetails {
    /// The layer1 chain 's genesis.
    pub layer1_genesis_hash: BlockHash,
    /// The layer2(producer) chain 's genesis.
    pub layer2_genesis_hash: BlockHash,
    /// The app id of the layer2 in layer1.
    pub layer2_app_id: u32,
    /// The verifier public key.
    pub verifier: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierStats {
    pub submission_period: AppPeriod,
    pub challenge_period: AppPeriod,
    pub waitting_submissions_count: u32,
    pub submitted: VerifierBlockInfos,
    pub challenged: VerifierBlockInfos,
}

impl Default for VerifierStats {
    fn default() -> Self {
        Self {
            submission_period: 0,
            challenge_period: 0,
            waitting_submissions_count: 0,
            submitted: VerifierBlockInfos::default(),
            challenged: VerifierBlockInfos::default(),
        }
    }
}

/// A couple of node statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeStats {
    pub peers: u64,
    pub txcount: u64,
}

// # A note about serialization/deserialization of types in this file:
//
// Some of the types here are sent to UI feeds. In an effort to keep the
// amount of bytes sent to a minimum, we have written custom serializers
// for those types.
//
// For testing purposes, it's useful to be able to deserialize from some
// of these types so that we can test message feed things, so custom
// deserializers exist to undo the work of the custom serializers.
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

impl<'de> Deserialize<'de> for NodeStats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (peers, txcount) = <(u64, u64)>::deserialize(deserializer)?;
        Ok(NodeStats { peers, txcount })
    }
}

/// Node IO details.
#[derive(Default)]
pub struct NodeIO {
    pub used_state_cache_size: MeanList<f32>,
}

impl Serialize for NodeIO {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(1)?;
        // This is "one-way": we can't deserialize again from this to a MeanList:
        tup.serialize_element(self.used_state_cache_size.slice())?;
        tup.end()
    }
}

/// Concise block details
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub hash: BlockHash,
    pub height: BlockNumber,
}

impl Block {
    pub fn zero() -> Self {
        Block {
            hash: BlockHash::from([0; 32]),
            height: 0,
        }
    }
}

/// Node hardware details.
#[derive(Default)]
pub struct NodeHardware {
    /// Upload uses means
    pub upload: MeanList<f64>,
    /// Download uses means
    pub download: MeanList<f64>,
    /// Stampchange uses means
    pub chart_stamps: MeanList<f64>,
}

impl Serialize for NodeHardware {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(3)?;
        // These are "one-way": we can't deserialize again from them to MeanLists:
        tup.serialize_element(self.upload.slice())?;
        tup.serialize_element(self.download.slice())?;
        tup.serialize_element(self.chart_stamps.slice())?;
        tup.end()
    }
}

/// Node location details
#[derive(Debug, Clone, PartialEq)]
pub struct NodeLocation {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Box<str>,
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

impl<'de> Deserialize<'de> for NodeLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (latitude, longitude, city) = <(f32, f32, Box<str>)>::deserialize(deserializer)?;
        Ok(NodeLocation {
            latitude,
            longitude,
            city,
        })
    }
}

/// Verbose block details
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockDetails {
    pub block: Block,
    pub block_time: u64,
    pub block_timestamp: u64,
    pub propagation_time: Option<u64>,
}

impl Default for BlockDetails {
    fn default() -> Self {
        BlockDetails {
            block: Block::zero(),
            block_timestamp: time::now(),
            block_time: 0,
            propagation_time: None,
        }
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

impl<'de> Deserialize<'de> for BlockDetails {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tup = <(u64, BlockHash, u64, u64, Option<u64>)>::deserialize(deserializer)?;
        Ok(BlockDetails {
            block: Block {
                height: tup.0,
                hash: tup.1,
            },
            block_time: tup.2,
            block_timestamp: tup.3,
            propagation_time: tup.4,
        })
    }
}
