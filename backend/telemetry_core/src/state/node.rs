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

use crate::find_location;
use common::node_message::SystemInterval;
use common::node_types::{
    AppPeriod, Block, BlockDetails, NodeDetails, NodeHardware, NodeHwBench, NodeIO, NodeLocation,
    NodeStats, Timestamp, VerifierBlockInfos, VerifierNodeDetails, VerifierStats,
};
use common::time;

/// Minimum time between block below broadcasting updates to the browser gets throttled, in ms.
const THROTTLE_THRESHOLD: u64 = 100;
/// Minimum time of intervals for block updates sent to the browser when throttled, in ms.
const THROTTLE_INTERVAL: u64 = 1000;

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Basic stats
    stats: NodeStats,
    /// Node IO stats
    io: NodeIO,
    /// Best block
    best: BlockDetails,
    /// Finalized block
    finalized: Block,
    /// Timer for throttling block updates
    throttle: u64,
    /// Hardware stats over time
    hardware: NodeHardware,
    /// Physical location details
    location: find_location::Location,
    /// Flag marking if the node is stale (not syncing or producing blocks)
    stale: bool,
    /// Unix timestamp for when node started up (falls back to connection time)
    startup_time: Option<Timestamp>,
    /// Hardware benchmark results for the node
    hwbench: Option<NodeHwBench>,
    /// The Static datas for verifier.
    verifier_details: Option<VerifierNodeDetails>,
    /// The State datas for verifier.
    verifier_stats: VerifierStats,
}

impl Node {
    pub fn new(mut details: NodeDetails) -> Self {
        let startup_time = details
            .startup_time
            .take()
            .and_then(|time| time.parse().ok());

        Node {
            details,
            stats: NodeStats::default(),
            io: NodeIO::default(),
            best: BlockDetails::default(),
            finalized: Block::zero(),
            throttle: 0,
            hardware: NodeHardware::default(),
            location: None,
            stale: false,
            startup_time,
            hwbench: None,
            verifier_details: None,
            verifier_stats: VerifierStats::default(),
        }
    }

    pub fn details(&self) -> &NodeDetails {
        &self.details
    }

    pub fn stats(&self) -> &NodeStats {
        &self.stats
    }

    pub fn io(&self) -> &NodeIO {
        &self.io
    }

    pub fn best(&self) -> &Block {
        &self.best.block
    }

    pub fn best_timestamp(&self) -> u64 {
        self.best.block_timestamp
    }

    pub fn finalized(&self) -> &Block {
        &self.finalized
    }

    pub fn hardware(&self) -> &NodeHardware {
        &self.hardware
    }

    pub fn location(&self) -> Option<&NodeLocation> {
        self.location.as_deref()
    }

    pub fn update_location(&mut self, location: find_location::Location) {
        self.location = location;
    }

    pub fn block_details(&self) -> &BlockDetails {
        &self.best
    }

    pub fn hwbench(&self) -> Option<&NodeHwBench> {
        self.hwbench.as_ref()
    }

    pub fn update_hwbench(&mut self, hwbench: NodeHwBench) -> Option<NodeHwBench> {
        self.hwbench.replace(hwbench)
    }

    pub fn update_block(&mut self, block: Block) -> bool {
        if block.height > self.best.block.height {
            self.stale = false;
            self.best.block = block;

            true
        } else {
            false
        }
    }

    pub fn update_details(
        &mut self,
        timestamp: u64,
        propagation_time: Option<u64>,
    ) -> Option<&BlockDetails> {
        self.best.block_time = timestamp - self.best.block_timestamp;
        self.best.block_timestamp = timestamp;
        self.best.propagation_time = propagation_time;

        if self.throttle < timestamp {
            if self.best.block_time <= THROTTLE_THRESHOLD {
                self.throttle = timestamp + THROTTLE_INTERVAL;
            }

            Some(&self.best)
        } else {
            None
        }
    }

    pub fn update_hardware(&mut self, interval: &SystemInterval) -> bool {
        let mut changed = false;

        if let Some(upload) = interval.bandwidth_upload {
            changed |= self.hardware.upload.push(upload);
        }
        if let Some(download) = interval.bandwidth_download {
            changed |= self.hardware.download.push(download);
        }
        self.hardware.chart_stamps.push(time::now() as f64);

        changed
    }

    pub fn update_stats(&mut self, interval: &SystemInterval) -> Option<&NodeStats> {
        let mut changed = false;

        if let Some(peers) = interval.peers {
            if peers != self.stats.peers {
                self.stats.peers = peers;
                changed = true;
            }
        }
        if let Some(txcount) = interval.txcount {
            if txcount != self.stats.txcount {
                self.stats.txcount = txcount;
                changed = true;
            }
        }

        if changed {
            Some(&self.stats)
        } else {
            None
        }
    }

    pub fn update_io(&mut self, interval: &SystemInterval) -> Option<&NodeIO> {
        let mut changed = false;

        if let Some(size) = interval.used_state_cache_size {
            changed |= self.io.used_state_cache_size.push(size);
        }

        if changed {
            Some(&self.io)
        } else {
            None
        }
    }

    pub fn update_finalized(&mut self, block: Block) -> Option<&Block> {
        if block.height > self.finalized.height {
            self.finalized = block;
            Some(self.finalized())
        } else {
            None
        }
    }

    pub fn update_stale(&mut self, threshold: u64) -> bool {
        if self.best.block_timestamp < threshold {
            self.stale = true;
        }

        self.stale
    }

    pub fn stale(&self) -> bool {
        self.stale
    }

    pub fn set_validator_address(&mut self, addr: Box<str>) -> bool {
        if self.details.validator.as_ref() == Some(&addr) {
            false
        } else {
            self.details.validator = Some(addr);
            true
        }
    }

    pub fn startup_time(&self) -> Option<Timestamp> {
        self.startup_time
    }

    pub fn verifier_details(&self) -> Option<&VerifierNodeDetails> {
        self.verifier_details.as_ref()
    }

    pub fn update_verifier_details(&mut self, details: VerifierNodeDetails) -> bool {
        if self.verifier_details.is_none() {
            self.verifier_details = Some(details);
            true
        } else {
            false
        }
    }

    pub fn verifier_submitted(&self) -> &VerifierBlockInfos {
        &self.verifier_stats.submitted
    }

    pub fn verifier_challenged(&self) -> &VerifierBlockInfos {
        &self.verifier_stats.challenged
    }

    pub fn update_verifier_submitted(&mut self, info: VerifierBlockInfos) -> bool {
        if self.verifier_stats.submitted.block_number < info.block_number {
            self.verifier_stats.submitted = info;
            true
        } else {
            false
        }
    }

    pub fn update_verifier_challenged(&mut self, info: VerifierBlockInfos) -> bool {
        if self.verifier_stats.challenged.block_number < info.block_number {
            self.verifier_stats.challenged = info;
            true
        } else {
            false
        }
    }

    pub fn verifier_submission_period(&self) -> AppPeriod {
        self.verifier_stats.submission_period
    }

    pub fn verifier_challenge_period(&self) -> AppPeriod {
        self.verifier_stats.challenge_period
    }

    pub fn update_verifier_submission_period(&mut self, period: AppPeriod) -> bool {
        if self.verifier_stats.submission_period < period {
            self.verifier_stats.submission_period = period;
            true
        } else {
            false
        }
    }

    pub fn update_verifier_challenge_period(&mut self, period: AppPeriod) -> bool {
        if self.verifier_stats.challenge_period < period {
            self.verifier_stats.challenge_period = period;
            true
        } else {
            false
        }
    }
}
