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

//! The structs and enums defined in this module are largely identical to those
//! we'll use elsewhere internally, but are kept separate so that the JSON structure
//! is defined (almost) from just this file, and we don't have to worry about breaking
//! compatibility with the input data when we make changes to our internal data
//! structures (for example, to support bincode better).
use super::hash::Hash;
use super::{
    ChainType, VerifierDetailsStats, VerifierNodeDetails, VerifierPeriodStats,
    VerifierProcessFinalityBlock,
};
use common::node_message as internal;
use common::node_types;
use serde::Deserialize;

/// This struct represents a telemetry message sent from a node as
/// a JSON payload. Since JSON is self describing, we can use attributes
/// like serde(untagged) and serde(flatten) without issue.
///
/// Internally, we want to minimise the amount of data sent from shards to
/// the core node. For that reason, we use a non-self-describing serialization
/// format like bincode, which doesn't support things like `[serde(flatten)]` (which
/// internally wants to serialize to a map of unknown length) or `[serde(tag/untagged)]`
/// (which relies on the data to know which variant to deserialize to.)
///
/// So, this can be converted fairly cheaply into an enum we'll use internally
/// which is compatible with formats like bincode.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum NodeMessage {
    V1 {
        #[serde(flatten)]
        payload: Payload,
    },
    V2 {
        id: NodeMessageId,
        payload: Payload,
        ts: Option<String>,
    },
}

impl From<NodeMessage> for internal::NodeMessage {
    fn from(msg: NodeMessage) -> Self {
        match msg {
            NodeMessage::V1 { payload } => internal::NodeMessage::V1 {
                payload: payload.into(),
            },
            NodeMessage::V2 { id, payload, .. } => internal::NodeMessage::V2 {
                id,
                payload: payload.into(),
            },
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "msg")]
pub enum Payload {
    #[serde(rename = "system.connected")]
    SystemConnected(SystemConnected),
    #[serde(rename = "system.interval")]
    SystemInterval(SystemInterval),
    #[serde(rename = "block.import")]
    BlockImport(Block),
    #[serde(rename = "notify.finalized")]
    NotifyFinalized(Finalized),
    #[serde(rename = "afg.authority_set")]
    AfgAuthoritySet(AfgAuthoritySet),
    #[serde(rename = "sysinfo.hwbench")]
    HwBench(NodeHwBench),
    #[serde(rename = "verifier.node_details")]
    VerifierNodeDetails(VerifierNodeDetails),
    #[serde(rename = "verifier.process_finality_block")]
    VerifierProcessFinalityBlock(VerifierProcessFinalityBlock),
    #[serde(rename = "verifier.details")]
    VerifierDetailsStats(VerifierDetailsStats),
    #[serde(rename = "verifier.period")]
    VerifierPeriodStats(VerifierPeriodStats),
}

impl From<Payload> for internal::Payload {
    fn from(msg: Payload) -> Self {
        match msg {
            Payload::SystemConnected(m) => internal::Payload::SystemConnected(m.into()),
            Payload::SystemInterval(m) => internal::Payload::SystemInterval(m.into()),
            Payload::BlockImport(m) => internal::Payload::BlockImport(m.into()),
            Payload::NotifyFinalized(m) => internal::Payload::NotifyFinalized(m.into()),
            Payload::AfgAuthoritySet(m) => internal::Payload::AfgAuthoritySet(m.into()),
            Payload::HwBench(m) => internal::Payload::HwBench(m.into()),
            Payload::VerifierNodeDetails(m) => internal::Payload::VerifierNodeDetails(m.into()),
            Payload::VerifierProcessFinalityBlock(m) => {
                internal::Payload::VerifierProcessFinalityBlock(m.into())
            }
            Payload::VerifierDetailsStats(m) => internal::Payload::VerifierDetailsStats(m.into()),
            Payload::VerifierPeriodStats(m) => internal::Payload::VerifierPeriodStats(m.into()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SystemConnected {
    pub genesis_hash: Hash,
    #[serde(flatten)]
    pub node: NodeDetails,
}

impl From<SystemConnected> for internal::SystemConnected {
    fn from(msg: SystemConnected) -> Self {
        internal::SystemConnected {
            genesis_hash: msg.genesis_hash.into(),
            node: msg.node.into(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SystemInterval {
    /// The chain type
    pub chain_type: Option<ChainType>,

    pub peers: Option<u64>,
    pub txcount: Option<u64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<Hash>,
    #[serde(flatten)]
    pub block: Option<Block>,
    pub used_state_cache_size: Option<f32>,
}

impl From<SystemInterval> for internal::SystemInterval {
    fn from(msg: SystemInterval) -> Self {
        internal::SystemInterval {
            chain_type: msg.chain_type,
            peers: msg.peers,
            txcount: msg.txcount,
            bandwidth_upload: msg.bandwidth_upload,
            bandwidth_download: msg.bandwidth_download,
            finalized_height: msg.finalized_height,
            finalized_hash: msg.finalized_hash.map(|h| h.into()),
            block: msg.block.map(|b| b.into()),
            used_state_cache_size: msg.used_state_cache_size,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Finalized {
    /// The chain type
    pub chain_type: Option<ChainType>,

    #[serde(rename = "best")]
    pub hash: Hash,
    pub height: Box<str>,
}

impl From<Finalized> for internal::Finalized {
    fn from(msg: Finalized) -> Self {
        internal::Finalized {
            chain_type: msg.chain_type,
            hash: msg.hash.into(),
            height: msg.height,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
}

impl From<AfgAuthoritySet> for internal::AfgAuthoritySet {
    fn from(msg: AfgAuthoritySet) -> Self {
        internal::AfgAuthoritySet {
            chain_type: None,
            authority_id: msg.authority_id,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Block {
    /// The chain type
    pub chain_type: Option<ChainType>,

    #[serde(rename = "best")]
    pub hash: Hash,
    pub height: BlockNumber,
}

impl From<Block> for node_types::Block {
    fn from(block: Block) -> Self {
        node_types::Block {
            hash: block.hash.into(),
            height: block.height,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeSysInfo {
    pub cpu: Option<Box<str>>,
    pub memory: Option<u64>,
    pub core_count: Option<u32>,
    pub linux_kernel: Option<Box<str>>,
    pub linux_distro: Option<Box<str>>,
    pub is_virtual_machine: Option<bool>,
}

impl From<NodeSysInfo> for node_types::NodeSysInfo {
    fn from(sysinfo: NodeSysInfo) -> Self {
        node_types::NodeSysInfo {
            cpu: sysinfo.cpu,
            memory: sysinfo.memory,
            core_count: sysinfo.core_count,
            linux_kernel: sysinfo.linux_kernel,
            linux_distro: sysinfo.linux_distro,
            is_virtual_machine: sysinfo.is_virtual_machine,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeHwBench {
    pub cpu_hashrate_score: u64,
    pub memory_memcpy_score: u64,
    pub disk_sequential_write_score: Option<u64>,
    pub disk_random_write_score: Option<u64>,
}

impl From<NodeHwBench> for node_types::NodeHwBench {
    fn from(hwbench: NodeHwBench) -> Self {
        node_types::NodeHwBench {
            cpu_hashrate_score: hwbench.cpu_hashrate_score,
            memory_memcpy_score: hwbench.memory_memcpy_score,
            disk_sequential_write_score: hwbench.disk_sequential_write_score,
            disk_random_write_score: hwbench.disk_random_write_score,
        }
    }
}

impl From<NodeHwBench> for internal::NodeHwBench {
    fn from(msg: NodeHwBench) -> Self {
        internal::NodeHwBench {
            cpu_hashrate_score: msg.cpu_hashrate_score,
            memory_memcpy_score: msg.memory_memcpy_score,
            disk_sequential_write_score: msg.disk_sequential_write_score,
            disk_random_write_score: msg.disk_random_write_score,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeDetails {
    pub chain: Box<str>,
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
    pub validator: Option<Box<str>>,
    pub network_id: node_types::NetworkId,
    pub startup_time: Option<Box<str>>,
    pub target_os: Option<Box<str>>,
    pub target_arch: Option<Box<str>>,
    pub target_env: Option<Box<str>>,
    pub sysinfo: Option<NodeSysInfo>,
    pub ip: Option<Box<str>>,
}

impl From<NodeDetails> for node_types::NodeDetails {
    fn from(mut details: NodeDetails) -> Self {
        // Migrate old-style `version` to the split metrics.
        // TODO: Remove this once everyone updates their nodes.
        if details.target_os.is_none()
            && details.target_arch.is_none()
            && details.target_env.is_none()
        {
            if let Some((version, target_arch, target_os, target_env)) =
                split_old_style_version(&details.version)
            {
                details.target_arch = Some(target_arch.into());
                details.target_os = Some(target_os.into());
                details.target_env = Some(target_env.into());
                details.version = version.into();
            }
        }

        node_types::NodeDetails {
            chain: details.chain,
            name: details.name,
            implementation: details.implementation,
            version: details.version,
            validator: details.validator,
            network_id: details.network_id,
            startup_time: details.startup_time,
            target_os: details.target_os,
            target_arch: details.target_arch,
            target_env: details.target_env,
            sysinfo: details.sysinfo.map(|sysinfo| sysinfo.into()),
            ip: details.ip,
        }
    }
}

type NodeMessageId = u64;
type BlockNumber = u64;

fn is_version_or_hash(name: &str) -> bool {
    name.bytes().all(|byte| {
        byte.is_ascii_digit()
            || byte == b'.'
            || byte == b'a'
            || byte == b'b'
            || byte == b'c'
            || byte == b'd'
            || byte == b'e'
            || byte == b'f'
    })
}

/// Split an old style version string into its version + target_arch + target_os + target_arch parts.
fn split_old_style_version(version_and_target: &str) -> Option<(&str, &str, &str, &str)> {
    // Old style versions are composed of the following parts:
    //    $version-$commit_hash-$arch-$os-$env
    // where $commit_hash and $env are optional.
    //
    // For example these are all valid:
    //   0.9.17-75dd6c7d0-x86_64-linux-gnu
    //   0.9.17-75dd6c7d0-x86_64-linux
    //   0.9.17-x86_64-linux-gnu
    //   0.9.17-x86_64-linux
    //   2.0.0-alpha.5-da487d19d-x86_64-linux

    let mut iter = version_and_target.rsplit('-').take(3).skip(2);

    // This will one of these: $arch, $commit_hash, $version
    let item = iter.next()?;

    let target_offset = if is_version_or_hash(item) {
        item.as_ptr() as usize + item.len() + 1
    } else {
        item.as_ptr() as usize
    } - version_and_target.as_ptr() as usize;

    let version = version_and_target.get(0..target_offset - 1)?;
    let mut target = version_and_target.get(target_offset..)?.split('-');
    let target_arch = target.next()?;
    let target_os = target.next()?;
    let target_env = target.next().unwrap_or("");

    Some((version, target_arch, target_os, target_env))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_v1() {
        let json = r#"{
            "msg":"notify.finalized",
            "level":"INFO",
            "ts":"2021-01-13T12:38:25.410794650+01:00",
            "best":"0x031c3521ca2f9c673812d692fc330b9a18e18a2781e3f9976992f861fd3ea0cb",
            "height":"50"
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V1 { .. },
            ),
            "message did not match variant V1",
        );
    }

    #[test]
    fn message_v2() {
        let json = r#"{
            "id":1,
            "ts":"2021-01-13T12:22:20.053527101+01:00",
            "payload":{
                "best":"0xcc41708573f2acaded9dd75e07dac2d4163d136ca35b3061c558d7a35a09dd8d",
                "height":"209",
                "msg":"notify.finalized"
            }
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V2 { .. },
            ),
            "message did not match variant V2",
        );
    }

    #[test]
    fn message_v2_tx_pool_import() {
        // We should happily ignore any fields we don't care about.
        let json = r#"{
            "id":1,
            "ts":"2021-01-13T12:22:20.053527101+01:00",
            "payload":{
                "foo":"Something",
                "bar":123,
                "wibble":"wobble",
                "msg":"block.import",
                "best":"0xcc41708573f2acaded9dd75e07dac2d4163d136ca35b3061c558d7a35a09dd8d",
                "height": 1234
            }
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V2 {
                    payload: Payload::BlockImport(Block { .. }),
                    ..
                },
            ),
            "message did not match the expected output",
        );
    }

    #[test]
    fn split_old_style_version_works() {
        let (version, target_arch, target_os, target_env) =
            split_old_style_version("0.9.17-75dd6c7d0-x86_64-linux-gnu").unwrap();
        assert_eq!(version, "0.9.17-75dd6c7d0");
        assert_eq!(target_arch, "x86_64");
        assert_eq!(target_os, "linux");
        assert_eq!(target_env, "gnu");

        let (version, target_arch, target_os, target_env) =
            split_old_style_version("0.9.17-75dd6c7d0-x86_64-linux").unwrap();
        assert_eq!(version, "0.9.17-75dd6c7d0");
        assert_eq!(target_arch, "x86_64");
        assert_eq!(target_os, "linux");
        assert_eq!(target_env, "");

        let (version, target_arch, target_os, target_env) =
            split_old_style_version("0.9.17-x86_64-linux-gnu").unwrap();
        assert_eq!(version, "0.9.17");
        assert_eq!(target_arch, "x86_64");
        assert_eq!(target_os, "linux");
        assert_eq!(target_env, "gnu");

        let (version, target_arch, target_os, target_env) =
            split_old_style_version("0.9.17-x86_64-linux").unwrap();
        assert_eq!(version, "0.9.17");
        assert_eq!(target_arch, "x86_64");
        assert_eq!(target_os, "linux");
        assert_eq!(target_env, "");

        let (version, target_arch, target_os, target_env) =
            split_old_style_version("2.0.0-alpha.5-da487d19d-x86_64-linux").unwrap();
        assert_eq!(version, "2.0.0-alpha.5-da487d19d");
        assert_eq!(target_arch, "x86_64");
        assert_eq!(target_os, "linux");
        assert_eq!(target_env, "");

        assert_eq!(split_old_style_version(""), None);
        assert_eq!(split_old_style_version("a"), None);
        assert_eq!(split_old_style_version("a-b"), None);
    }
}
