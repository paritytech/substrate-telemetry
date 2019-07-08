use crate::types::{NodeId, NodeDetails, NodeStats, NodeHardware, NodeLocation, BlockNumber};
use crate::util::MeanList;

pub mod message;
pub mod connector;

use message::{NodeMessage, Details, Block};
use std::time::{SystemTime, Instant, Duration};

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Basic stats
    stats: NodeStats,
    /// Best block
    best: Block,
    /// Timestamp of best block
    block_timestamp: Instant,
    /// Block time delta
    block_time: u64,
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
}

impl Node {
    pub fn new(details: NodeDetails) -> Self {
        Node {
            details,
            stats: NodeStats {
                txcount: 0,
                peers: 0,
            },
            best: Block::zero(),
            block_timestamp: Instant::now(),
            block_time: 0,
            cpu: MeanList::new(),
            memory: MeanList::new(),
            upload: MeanList::new(),
            download: MeanList::new(),
            chart_stamps: MeanList::new(),
        }
    }

    pub fn details(&self) -> &NodeDetails {
        &self.details
    }

    pub fn stats(&self) -> &NodeStats {
        &self.stats
    }

    pub fn best(&self) -> &Block {
        &self.best
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

    pub fn location(&self) -> NodeLocation {
        (0.0, 0.0, "")
    }

    pub fn block_time(&self) -> u64 {
        self.block_time
    }

    pub fn update_block_time(&mut self, block_height: BlockNumber, timestamp: Instant) {
        if block_height > self.best.height {
            self.block_time = (timestamp - self.block_timestamp).as_millis() as u64;
            self.block_timestamp = timestamp;
        }
    }

    pub fn update(&mut self, msg: NodeMessage) {
        if let Some(block) = msg.details.best_block() {
            if block.height > self.best.height {
                self.best = *block;
            }
        }

        match msg.details {
            Details::SystemInterval(ref interval) => {
                self.stats = interval.stats;
                if let Some(cpu) = interval.cpu {
                    self.cpu.push(cpu);
                }
                if let Some(memory) = interval.memory {
                    self.memory.push(memory);
                }
                if let Some(upload) = interval.bandwidth_upload {
                    self.upload.push(upload);
                }
                if let Some(download) = interval.bandwidth_download {
                    self.download.push(download);
                }
                let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or(Duration::from_secs(0)).as_millis() as f64;
                self.chart_stamps.push(timestamp);
            }
            _ => ()
        }
    }
}
