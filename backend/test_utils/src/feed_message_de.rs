use anyhow::Context;
use common::node_types::{
    BlockDetails, BlockHash, BlockNumber, NodeLocation, NodeStats, Timestamp,
};
use serde_json::value::RawValue;

#[derive(Debug, PartialEq)]
pub enum FeedMessage {
    Version(usize),
    BestBlock {
        block_number: BlockNumber,
        timestamp: Timestamp,
        avg_block_time: Option<u64>,
    },
    BestFinalized {
        block_number: BlockNumber,
        block_hash: BlockHash,
    },
    AddedNode {
        node_id: usize,
        node: NodeDetails,
        stats: NodeStats,
        // io: NodeIO, // can't losslessly deserialize
        // hardware: NodeHardware, // can't losslessly deserialize
        block_details: BlockDetails,
        location: Option<NodeLocation>,
        startup_time: Option<Timestamp>,
    },
    RemovedNode {
        node_id: usize,
    },
    LocatedNode {
        node_id: usize,
        lat: f32,
        long: f32,
        city: String,
    },
    ImportedBlock {
        node_id: usize,
        block_details: BlockDetails,
    },
    FinalizedBlock {
        node_id: usize,
        block_number: BlockNumber,
        block_hash: BlockHash,
    },
    NodeStatsUpdate {
        node_id: usize,
        stats: NodeStats,
    },
    Hardware {
        node_id: usize,
        // hardware: NodeHardware, // Can't losslessly deserialize
    },
    TimeSync {
        time: Timestamp,
    },
    AddedChain {
        name: String,
        node_count: usize,
    },
    RemovedChain {
        name: String,
    },
    SubscribedTo {
        name: String,
    },
    UnsubscribedFrom {
        name: String,
    },
    Pong {
        msg: String,
    },
    AfgFinalized {
        address: String,
        block_number: BlockNumber,
        block_hash: BlockHash,
    },
    AfgReceivedPrevote {
        address: String,
        block_number: BlockNumber,
        block_hash: BlockHash,
        voter: Option<String>,
    },
    AfgReceivedPrecommit {
        address: String,
        block_number: BlockNumber,
        block_hash: BlockHash,
        voter: Option<String>,
    },
    AfgAuthoritySet {
        // Not used currently; not sure what "address" params are:
        a1: String,
        a2: String,
        a3: String,
        block_number: BlockNumber,
        block_hash: BlockHash,
    },
    StaleNode {
        node_id: usize,
    },
    NodeIOUpdate {
        node_id: usize,
        // details: NodeIO, // can't losslessly deserialize
    },
    /// A "special" case when we don't know how to decode an action:
    UnknownValue {
        action: u8,
        value: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct NodeDetails {
    pub name: String,
    pub implementation: String,
    pub version: String,
    pub validator: Option<String>,
    pub network_id: Option<String>,
}

impl FeedMessage {
    /// Decode a slice of bytes into a vector of feed messages
    pub fn from_bytes(bytes: &[u8]) -> Result<Vec<FeedMessage>, anyhow::Error> {
        let v: Vec<&RawValue> = serde_json::from_slice(bytes)?;

        let mut feed_messages = vec![];
        for raw_keyval in v.chunks(2) {
            let raw_key = raw_keyval[0];
            let raw_val = raw_keyval[1];
            let action: u8 = serde_json::from_str(raw_key.get())?;
            let msg = FeedMessage::decode(action, raw_val)
                .with_context(|| format!("Failed to decode message with action {}", action))?;

            feed_messages.push(msg);
        }

        Ok(feed_messages)
    }

    // Deserialize the feed message to a value based on the "action" key
    fn decode(action: u8, raw_val: &RawValue) -> Result<FeedMessage, anyhow::Error> {
        let feed_message = match action {
            // Version:
            0 => {
                let version = serde_json::from_str(raw_val.get())?;
                FeedMessage::Version(version)
            }
            // BestBlock
            1 => {
                let (block_number, timestamp, avg_block_time) =
                    serde_json::from_str(raw_val.get())?;
                FeedMessage::BestBlock {
                    block_number,
                    timestamp,
                    avg_block_time,
                }
            }
            // BestFinalized
            2 => {
                let (block_number, block_hash) = serde_json::from_str(raw_val.get())?;
                FeedMessage::BestFinalized {
                    block_number,
                    block_hash,
                }
            }
            // AddNode
            3 => {
                let (
                    node_id,
                    (name, implementation, version, validator, network_id),
                    stats,
                    io,
                    hardware,
                    block_details,
                    location,
                    startup_time,
                ) = serde_json::from_str(raw_val.get())?;

                // Give these two types but don't use the results:
                let (_, _): (&RawValue, &RawValue) = (io, hardware);

                FeedMessage::AddedNode {
                    node_id,
                    node: NodeDetails {
                        name,
                        implementation,
                        version,
                        validator,
                        network_id,
                    },
                    stats,
                    block_details,
                    location,
                    startup_time,
                }
            }
            // RemoveNode
            4 => {
                let node_id = serde_json::from_str(raw_val.get())?;
                FeedMessage::RemovedNode { node_id }
            }
            // LocatedNode
            5 => {
                let (node_id, lat, long, city) = serde_json::from_str(raw_val.get())?;
                FeedMessage::LocatedNode {
                    node_id,
                    lat,
                    long,
                    city,
                }
            }
            // ImportedBlock
            6 => {
                let (node_id, block_details) = serde_json::from_str(raw_val.get())?;
                FeedMessage::ImportedBlock {
                    node_id,
                    block_details,
                }
            }
            // FinalizedBlock
            7 => {
                let (node_id, block_number, block_hash) = serde_json::from_str(raw_val.get())?;
                FeedMessage::FinalizedBlock {
                    node_id,
                    block_number,
                    block_hash,
                }
            }
            // NodeStatsUpdate
            8 => {
                let (node_id, stats) = serde_json::from_str(raw_val.get())?;
                FeedMessage::NodeStatsUpdate { node_id, stats }
            }
            // Hardware
            9 => {
                let (node_id, _hardware): (_, &RawValue) = serde_json::from_str(raw_val.get())?;
                FeedMessage::Hardware { node_id }
            }
            // TimeSync
            10 => {
                let time = serde_json::from_str(raw_val.get())?;
                FeedMessage::TimeSync { time }
            }
            // AddedChain
            11 => {
                let (name, node_count) = serde_json::from_str(raw_val.get())?;
                FeedMessage::AddedChain { name, node_count }
            }
            // RemovedChain
            12 => {
                let name = serde_json::from_str(raw_val.get())?;
                FeedMessage::RemovedChain { name }
            }
            // SubscribedTo
            13 => {
                let name = serde_json::from_str(raw_val.get())?;
                FeedMessage::SubscribedTo { name }
            }
            // UnsubscribedFrom
            14 => {
                let name = serde_json::from_str(raw_val.get())?;
                FeedMessage::UnsubscribedFrom { name }
            }
            // Pong
            15 => {
                let msg = serde_json::from_str(raw_val.get())?;
                FeedMessage::Pong { msg }
            }
            // AfgFinalized
            16 => {
                let (address, block_number, block_hash) = serde_json::from_str(raw_val.get())?;
                FeedMessage::AfgFinalized {
                    address,
                    block_number,
                    block_hash,
                }
            }
            // AfgReceivedPrevote
            17 => {
                let (address, block_number, block_hash, voter) =
                    serde_json::from_str(raw_val.get())?;
                FeedMessage::AfgReceivedPrevote {
                    address,
                    block_number,
                    block_hash,
                    voter,
                }
            }
            // AfgReceivedPrecommit
            18 => {
                let (address, block_number, block_hash, voter) =
                    serde_json::from_str(raw_val.get())?;
                FeedMessage::AfgReceivedPrecommit {
                    address,
                    block_number,
                    block_hash,
                    voter,
                }
            }
            // AfgAuthoritySet
            19 => {
                let (a1, a2, a3, block_number, block_hash) = serde_json::from_str(raw_val.get())?;
                FeedMessage::AfgAuthoritySet {
                    a1,
                    a2,
                    a3,
                    block_number,
                    block_hash,
                }
            }
            // StaleNode
            20 => {
                let node_id = serde_json::from_str(raw_val.get())?;
                FeedMessage::StaleNode { node_id }
            }
            // NodeIOUpdate
            21 => {
                // ignore NodeIO for now:
                let (node_id, _node_io): (_, &RawValue) = serde_json::from_str(raw_val.get())?;
                FeedMessage::NodeIOUpdate { node_id }
            }
            // A catchall for messages we don't know/care about yet:
            _ => {
                let value = raw_val.to_string();
                FeedMessage::UnknownValue { action, value }
            }
        };

        Ok(feed_message)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn decode_remove_node_msg() {
        // "remove chain ''":
        let msg = r#"[12,""]"#;

        assert_eq!(
            FeedMessage::from_bytes(msg.as_bytes()).unwrap(),
            vec![FeedMessage::RemovedChain {
                name: "".to_owned()
            }]
        );
    }

    #[test]
    fn decode_remove_then_add_node_msg() {
        // "remove chain '', then add chain 'Local Testnet' with 1 node":
        let msg = r#"[12,"",11,["Local Testnet",1]]"#;

        assert_eq!(
            FeedMessage::from_bytes(msg.as_bytes()).unwrap(),
            vec![
                FeedMessage::RemovedChain {
                    name: "".to_owned()
                },
                FeedMessage::AddedChain {
                    name: "Local Testnet".to_owned(),
                    node_count: 1
                },
            ]
        );
    }
}
