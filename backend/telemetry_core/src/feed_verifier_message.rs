//! This module provides a way of encoding the various messages that we'll
//! send to subscribed feeds for alt-verifier (browsers).

use crate::feed_message::*;
use common::node_types::{
    AppPeriod, BlockDetails, BlockHash, BlockNumber, NodeIO, NodeStats, VerifierBlockInfos,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierPeriodStats {
    pub submission: Option<AppPeriod>,
    pub challenge: Option<AppPeriod>,
}

#[derive(Serialize)]
pub struct SubmittedBlock(pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct ChallengedBlock(pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct Period(pub AppPeriod, pub AppPeriod);

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
pub struct VerifierNodeSubmittedBlockStats<'a>(pub FeedNodeId, pub &'a VerifierBlockInfos);

#[derive(Serialize)]
pub struct VerifierNodeChallengedBlockStats<'a>(pub FeedNodeId, pub &'a VerifierBlockInfos);

#[derive(Serialize)]
pub struct VerifierNodeSubmissionPeriodStats(pub FeedNodeId, pub AppPeriod);

#[derive(Serialize)]
pub struct VerifierNodeChallengePeriodStats(pub FeedNodeId, pub AppPeriod);
