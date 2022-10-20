//! This module provides a way of encoding the various messages that we'll
//! send to subscribed feeds for alt-verifier (browsers).

use crate::feed_message::*;
use common::node_types::{BlockDetails, BlockHash, BlockNumber, NodeIO, NodeStats};
use serde::{Deserialize, Serialize};

pub type AppPeriod = u32;

pub type DigestHash = primitive_types::H256;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierPeriodStats {
    pub submission: AppPeriod,
    pub challenge: AppPeriod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierDetailsStats {
    pub waitting_submissions_count: u32,
    pub submitted_digest: DigestHash,
    pub submitted_block_number: u64,
    pub submitted_block_hash: BlockHash,
    pub challenged_digest: DigestHash,
    pub challenged_block_number: u64,
    pub challenged_block_hash: BlockHash,
}

#[derive(Serialize)]
pub struct CommittedBlock(pub FeedNodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct ChallengedBlock(pub FeedNodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct Period<'a>(pub FeedNodeId, pub &'a VerifierPeriodStats);

#[derive(Serialize)]
pub struct Layer1ImportedBlock<'a>(pub FeedNodeId, pub &'a BlockDetails);

#[derive(Serialize)]
pub struct Layer1FinalizedBlock(pub FeedNodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct Layer1NodeStatsUpdate<'a>(pub FeedNodeId, pub &'a NodeStats);

#[derive(Serialize)]
pub struct Layer1NodeIOUpdate<'a>(pub FeedNodeId, pub &'a NodeIO);

#[derive(Serialize)]
pub struct Layer2ImportedBlock<'a>(pub FeedNodeId, pub &'a BlockDetails);

#[derive(Serialize)]
pub struct Layer2FinalizedBlock(pub FeedNodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct Layer2NodeStatsUpdate<'a>(pub FeedNodeId, pub &'a NodeStats);

#[derive(Serialize)]
pub struct Layer2NodeIOUpdate<'a>(pub FeedNodeId, pub &'a NodeIO);

#[derive(Serialize)]
pub struct VerifierStats<'a>(pub FeedNodeId, pub &'a VerifierDetailsStats);
