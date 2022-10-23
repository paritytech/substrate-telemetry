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

//! This module provides a way of encoding the various messages that we'll
//! send to subscribed feeds (browsers).

use serde::Serialize;

pub use crate::feed_verifier_message::*;

use crate::state::Node;
use common::node_types::{
    BlockDetails, BlockHash, BlockNumber, NodeHardware, NodeIO, NodeStats, Timestamp,
};
use serde_json::to_writer;

pub type FeedNodeId = usize;

pub trait FeedMessage {
    const ACTION: u8;
}

pub trait FeedMessageWrite: FeedMessage {
    fn write_to_feed(&self, ser: &mut FeedMessageSerializer);
}

impl<T> FeedMessageWrite for T
where
    T: FeedMessage + Serialize,
{
    fn write_to_feed(&self, ser: &mut FeedMessageSerializer) {
        ser.write(self)
    }
}

pub struct FeedMessageSerializer {
    /// Current buffer.
    buffer: Vec<u8>,
}

const BUFCAP: usize = 128;

impl FeedMessageSerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(BUFCAP),
        }
    }

    pub fn push<Message>(&mut self, msg: Message)
    where
        Message: FeedMessageWrite,
    {
        let glue = match self.buffer.len() {
            0 => b'[',
            _ => b',',
        };

        self.buffer.push(glue);
        self.write(&Message::ACTION);
        self.buffer.push(b',');
        msg.write_to_feed(self);
    }

    fn write<S>(&mut self, value: &S)
    where
        S: Serialize,
    {
        let _ = to_writer(&mut self.buffer, value);
    }

    /// Return the bytes that we've serialized so far, consuming the serializer.
    pub fn into_finalized(mut self) -> Option<bytes::Bytes> {
        if self.buffer.is_empty() {
            return None;
        }

        self.buffer.push(b']');
        Some(self.buffer.into())
    }
}

macro_rules! actions {
    ($($action:literal: $t:ty,)*) => {
        $(
            impl FeedMessage for $t {
                const ACTION: u8 = $action;
            }
        )*
    }
}

actions! {
     0: Version,
     1: BestBlock,
     2: BestFinalized,
     3: AddedNode<'_>,
     4: RemovedNode,
     5: LocatedNode<'_>,
     6: ImportedBlock<'_>,
     7: FinalizedBlock,
     8: NodeStatsUpdate<'_>,
     9: Hardware<'_>,
    10: TimeSync,
    11: AddedChain<'_>,
    12: RemovedChain,
    13: SubscribedTo,
    14: UnsubscribedFrom,
    15: Pong<'_>,
    // Note; some now-unused messages were removed between IDs 15 and 20.
    // We maintain existing IDs for backward compatibility.
    20: StaleNode,
    21: NodeIOUpdate<'_>,
    22: ChainStatsUpdate<'_>,
    // The msgs for verifier messages
    61: SubmittedBlock,
    62: ChallengedBlock,
    63: Period,
    // The msgs for verifier node messages
    71: Layer1ImportedBlock<'_>,
    72: Layer1FinalizedBlock,
    73: Layer1NodeStatsUpdate<'_>,
    74: Layer1NodeIOUpdate<'_>,
    75: Layer2ImportedBlock<'_>,
    76: Layer2FinalizedBlock,
    77: Layer2NodeStatsUpdate<'_>,
    78: Layer2NodeIOUpdate<'_>,
    81: VerifierNodeSubmittedBlockStats<'_>,
    82: VerifierNodeChallengedBlockStats<'_>,
    83: VerifierNodeSubmissionPeriodStats,
    84: VerifierNodeChallengePeriodStats,
}

#[derive(Serialize)]
pub struct Version(pub usize);

#[derive(Serialize)]
pub struct BestBlock(pub BlockNumber, pub Timestamp, pub Option<u64>);

#[derive(Serialize)]
pub struct BestFinalized(pub BlockNumber, pub BlockHash);

pub struct AddedNode<'a>(pub FeedNodeId, pub &'a Node);

#[derive(Serialize)]
pub struct RemovedNode(pub FeedNodeId);

#[derive(Serialize)]
pub struct LocatedNode<'a>(pub FeedNodeId, pub f32, pub f32, pub &'a str);

#[derive(Serialize)]
pub struct ImportedBlock<'a>(pub FeedNodeId, pub &'a BlockDetails);

#[derive(Serialize)]
pub struct FinalizedBlock(pub FeedNodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct NodeStatsUpdate<'a>(pub FeedNodeId, pub &'a NodeStats);

#[derive(Serialize)]
pub struct NodeIOUpdate<'a>(pub FeedNodeId, pub &'a NodeIO);

#[derive(Serialize)]
pub struct Hardware<'a>(pub FeedNodeId, pub &'a NodeHardware);

#[derive(Serialize)]
pub struct TimeSync(pub u64);

#[derive(Serialize)]
pub struct AddedChain<'a>(pub &'a str, pub BlockHash, pub usize);

#[derive(Serialize)]
pub struct RemovedChain(pub BlockHash);

#[derive(Serialize)]
pub struct SubscribedTo(pub BlockHash);

#[derive(Serialize)]
pub struct UnsubscribedFrom(pub BlockHash);

#[derive(Serialize)]
pub struct Pong<'a>(pub &'a str);

#[derive(Serialize)]
pub struct StaleNode(pub FeedNodeId);

impl FeedMessageWrite for AddedNode<'_> {
    fn write_to_feed(&self, ser: &mut FeedMessageSerializer) {
        let AddedNode(nid, node) = self;

        let details = node.details();
        let details = (
            &details.name,
            &details.implementation,
            &details.version,
            &details.validator,
            &details.network_id,
            &details.ip,
        );

        ser.write(&(
            nid,
            details,
            node.stats(),
            node.io(),
            node.hardware(),
            node.block_details(),
            &node.location(),
            &node.startup_time(),
        ));
    }
}

#[derive(Serialize)]
pub struct ChainStatsUpdate<'a>(pub &'a ChainStats);

#[derive(Serialize, PartialEq, Eq, Default)]
pub struct Ranking<K> {
    pub list: Vec<(K, u64)>,
    pub other: u64,
    pub unknown: u64,
}

#[derive(Serialize, PartialEq, Eq, Default)]
pub struct ChainStats {
    pub version: Ranking<String>,
    pub target_os: Ranking<String>,
    pub target_arch: Ranking<String>,
    pub cpu: Ranking<String>,
    pub memory: Ranking<(u32, Option<u32>)>,
    pub core_count: Ranking<u32>,
    pub linux_kernel: Ranking<String>,
    pub linux_distro: Ranking<String>,
    pub is_virtual_machine: Ranking<bool>,
    pub cpu_hashrate_score: Ranking<(u32, Option<u32>)>,
    pub memory_memcpy_score: Ranking<(u32, Option<u32>)>,
    pub disk_sequential_write_score: Ranking<(u32, Option<u32>)>,
    pub disk_random_write_score: Ranking<(u32, Option<u32>)>,
}
