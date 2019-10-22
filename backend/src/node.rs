use crate::types::{NodeId, NodeDetails, NodeStats, NodeHardware, NodeLocation, BlockDetails, Block};
use crate::util::{MeanList, now};

pub mod message;
pub mod connector;

use message::SystemInterval;

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Basic stats
    stats: NodeStats,
    /// Best block
    best: BlockDetails,
    /// Finalized block
    finalized: Block,
    /// CPU use means
    cpu: MeanList<f32>,
    /// Memory use means
    memory: MeanList<f32>,
    /// Upload uses means
    upload: MeanList<f64>,
    /// Download uses means
    download: MeanList<f64>,
    /// Stampchange uses means
    chart_stamps: MeanList<f64>,
    /// Physical location details
    location: Option<NodeLocation>,
}

impl Node {
    pub fn new(details: NodeDetails) -> Self {
        Node {
            details,
            stats: NodeStats {
                txcount: 0,
                peers: 0,
            },
            best: BlockDetails {
                block: Block::zero(),
                block_timestamp: now(),
                block_time: 0,
                propagation_time: 0,
            },
            finalized: Block::zero(),
            cpu: MeanList::new(),
            memory: MeanList::new(),
            upload: MeanList::new(),
            download: MeanList::new(),
            chart_stamps: MeanList::new(),
            location: None,
        }
    }

    pub fn details(&self) -> &NodeDetails {
        &self.details
    }

    pub fn stats(&self) -> &NodeStats {
        &self.stats
    }

    pub fn best(&self) -> &Block {
        &self.best.block
    }

    pub fn finalized(&self) -> &Block {
        &self.finalized
    }

    pub fn hardware(&self) -> NodeHardware {
        (
            self.memory.slice(),
            self.cpu.slice(),
            self.upload.slice(),
            self.download.slice(),
            self.chart_stamps.slice(),
        )
    }

    pub fn location(&self) -> Option<&NodeLocation> {
        self.location.as_ref()
    }

    pub fn update_location(&mut self, location: NodeLocation) {
        self.location = Some(location);
    }

    pub fn block_time(&self) -> u64 {
        self.best.block_time
    }

    pub fn block_details(&self) -> &BlockDetails {
        &self.best
    }

    pub fn update_block(&mut self, block: Block, timestamp: u64, propagation_time: u64) {
        if block.height > self.best.block.height {
            self.best.block = block;
            self.best.block_time = timestamp - self.best.block_timestamp;
            self.best.block_timestamp = timestamp;
            self.best.propagation_time = propagation_time;
        }
    }

    pub fn update_hardware(&mut self, interval: &SystemInterval) -> bool {
        let mut changed = false;

        self.stats = interval.stats;
        if let Some(cpu) = interval.cpu {
            changed |= self.cpu.push(cpu);
        }
        if let Some(memory) = interval.memory {
            changed |= self.memory.push(memory);
        }
        if let Some(upload) = interval.bandwidth_upload {
            changed |= self.upload.push(upload);
        }
        if let Some(download) = interval.bandwidth_download {
            changed |= self.download.push(download);
        }
        self.chart_stamps.push(now() as f64);

        changed
    }

    pub fn update_finalized(&mut self, interval: &SystemInterval) -> Option<&Block> {
        if let (Some(height), Some(hash)) = (interval.finalized_height, interval.finalized_hash) {
            if height > self.finalized.height {
                self.finalized.height = height;
                self.finalized.hash = hash;
                return Some(self.finalized());
            }
        }

        None
    }
}
