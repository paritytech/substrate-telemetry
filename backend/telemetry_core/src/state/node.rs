use crate::find_location;
use common::node_message::SystemInterval;
use common::node_types::{
    Block, BlockDetails, NodeDetails, NodeHardware, NodeIO, NodeLocation, NodeStats, Timestamp,
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

    pub fn set_validator_address(&mut self, addr: Box<str>) {
        self.details.validator = Some(addr);
    }

    pub fn startup_time(&self) -> Option<Timestamp> {
        self.startup_time
    }
}
