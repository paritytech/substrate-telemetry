use actix::prelude::*;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::aggregator::{Aggregator, DropChain, NodeCount, NodeSource, RenameChain};
use crate::feed::connector::{FeedConnector, FeedId, Subscribed, Unsubscribed};
use crate::feed::{self, FeedMessageSerializer};
use crate::node::Node;
use shared::types::{Block, NodeDetails, NodeId, NodeLocation, Timestamp};
use shared::util::{now, DenseMap, NumStats};
use shared::node::Payload;

const STALE_TIMEOUT: u64 = 2 * 60 * 1000; // 2 minutes

pub type ChainId = usize;
pub type Label = Arc<str>;

pub struct Chain {
    cid: ChainId,
    /// Who to inform if the Chain drops itself
    aggregator: Addr<Aggregator>,
    /// Label of this chain, along with count of nodes that use this label
    label: (Label, usize),
    /// Dense mapping of NodeId -> Node
    nodes: DenseMap<Node>,
    /// Dense mapping of FeedId -> Addr<FeedConnector>,
    feeds: DenseMap<Addr<FeedConnector>>,
    /// Mapping of FeedId -> Addr<FeedConnector> for feeds requiring finality info,
    finality_feeds: FxHashMap<FeedId, Addr<FeedConnector>>,
    /// Best block
    best: Block,
    /// Finalized block
    finalized: Block,
    /// Block times history, stored so we can calculate averages
    block_times: NumStats<u64>,
    /// Calculated average block time
    average_block_time: Option<u64>,
    /// Message serializer
    serializer: FeedMessageSerializer,
    /// When the best block first arrived
    timestamp: Option<Timestamp>,
    /// Some nodes might manifest a different label, note them here
    labels: HashMap<Label, usize>,
}

impl Chain {
    pub fn new(cid: ChainId, aggregator: Addr<Aggregator>, label: Label) -> Self {
        log::info!("[{}] Created", label);

        Chain {
            cid,
            aggregator,
            label: (label, 0),
            nodes: DenseMap::new(),
            feeds: DenseMap::new(),
            finality_feeds: FxHashMap::default(),
            best: Block::zero(),
            finalized: Block::zero(),
            block_times: NumStats::new(50),
            average_block_time: None,
            serializer: FeedMessageSerializer::new(),
            timestamp: None,
            labels: HashMap::default(),
        }
    }

    fn increment_label_count(&mut self, label: &str) {
        let count = match self.labels.get_mut(label) {
            Some(count) => {
                *count += 1;
                *count
            }
            None => {
                self.labels.insert(label.into(), 1);
                1
            }
        };

        if &*self.label.0 == label {
            self.label.1 += 1;
        } else if count > self.label.1 {
            self.rename(label.into(), count);
        }
    }

    fn decrement_label_count(&mut self, label: &str) {
        match self.labels.get_mut(label) {
            Some(count) => *count -= 1,
            None => return,
        };

        if &*self.label.0 == label {
            self.label.1 -= 1;

            for (label, &count) in self.labels.iter() {
                if count > self.label.1 {
                    let label: Arc<_> = label.clone();
                    self.rename(label, count);
                    break;
                }
            }
        }
    }

    fn rename(&mut self, label: Label, count: usize) {
        self.label = (label, count);

        self.aggregator
            .do_send(RenameChain(self.cid, self.label.0.clone()));
    }

    fn broadcast(&mut self) {
        if let Some(msg) = self.serializer.finalize() {
            for (_, feed) in self.feeds.iter() {
                feed.do_send(msg.clone());
            }
        }
    }

    fn broadcast_finality(&mut self) {
        if let Some(msg) = self.serializer.finalize() {
            for feed in self.finality_feeds.values() {
                feed.do_send(msg.clone());
            }
        }
    }

    /// Triggered when the number of nodes in this chain has changed, Aggregator will
    /// propagate new counts to all connected feeds
    fn update_count(&self) {
        self.aggregator
            .do_send(NodeCount(self.cid, self.nodes.len()));
    }

    /// Check if the chain is stale (has not received a new best block in a while).
    /// If so, find a new best block, ignoring any stale nodes and marking them as such.
    fn update_stale_nodes(&mut self, now: u64) {
        let threshold = now - STALE_TIMEOUT;
        let timestamp = match self.timestamp {
            Some(ts) => ts,
            None => return,
        };

        if timestamp > threshold {
            // Timestamp is in range, nothing to do
            return;
        }

        let mut best = Block::zero();
        let mut finalized = Block::zero();
        let mut timestamp = None;

        for (nid, node) in self.nodes.iter_mut() {
            if !node.update_stale(threshold) {
                if node.best().height > best.height {
                    best = *node.best();
                    timestamp = Some(node.best_timestamp());
                }

                if node.finalized().height > finalized.height {
                    finalized = *node.finalized();
                }
            } else {
                self.serializer.push(feed::StaleNode(nid));
            }
        }

        if self.best.height != 0 || self.finalized.height != 0 {
            self.best = best;
            self.finalized = finalized;
            self.block_times.reset();
            self.timestamp = timestamp;

            self.serializer.push(feed::BestBlock(
                self.best.height,
                timestamp.unwrap_or(now),
                None,
            ));
            self.serializer
                .push(feed::BestFinalized(finalized.height, finalized.hash));
        }
    }
}

impl Actor for Chain {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        self.aggregator.do_send(DropChain(self.cid));

        for (_, feed) in self.feeds.iter() {
            feed.do_send(Unsubscribed)
        }
    }
}

/// Message sent from the Aggregator to the Chain when new Node is connected
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddNode {
    /// Details of the node being added to the aggregator
    pub node: NodeDetails,
    /// Source from which this node is being added (Direct | Shard)
    pub source: NodeSource,
}

/// Message sent from the NodeConnector to the Chain when it receives new telemetry data
#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateNode {
    pub nid: NodeId,
    pub payload: Payload,
}

/// Message sent from the NodeConnector to the Chain when the connector disconnects
#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveNode(pub NodeId);

/// Message sent from the Aggregator to the Chain when the connector wants to subscribe to that chain
#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe(pub Addr<FeedConnector>);

/// Message sent from the FeedConnector before it subscribes to a new chain, or if it disconnects
#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe(pub FeedId);

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendFinality(pub FeedId);

#[derive(Message)]
#[rtype(result = "()")]
pub struct NoMoreFinality(pub FeedId);

/// Message sent from the NodeConnector to the Chain when it receives location data
#[derive(Message)]
#[rtype(result = "()")]
pub struct LocateNode {
    pub nid: NodeId,
    pub location: Arc<NodeLocation>,
}

impl NodeSource {
    pub fn init(self, nid: NodeId, chain: Addr<Chain>) -> bool {
        match self {
            NodeSource::Direct { conn_id, node_connector } => {
                node_connector
                    .try_send(crate::node::connector::Initialize {
                        nid,
                        conn_id,
                        chain,
                    })
                    .is_ok()
            },
            NodeSource::Shard { sid, shard_connector } => {
                shard_connector
                    .try_send(crate::shard::connector::Initialize {
                        nid,
                        sid,
                        chain,
                    })
                    .is_ok()
            }
        }
    }
}

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode {
            node,
            source,
        } = msg;
        log::trace!(target: "Chain::AddNode", "New node connected. Chain '{}', node count goes from {} to {}", node.chain, self.nodes.len(), self.nodes.len() + 1);
        self.increment_label_count(&node.chain);

        let nid = self.nodes.add(Node::new(node));
        let chain = ctx.address();

        if source.init(nid, chain) {
            self.nodes.remove(nid);
        } else if let Some(node) = self.nodes.get(nid) {
            self.serializer.push(feed::AddedNode(nid, node));
            self.broadcast();
        }

        self.update_count();
    }
}

impl Chain {
    fn handle_block(&mut self, block: &Block, nid: NodeId) {
        let mut propagation_time = None;
        let now = now();
        let nodes_len = self.nodes.len();

        self.update_stale_nodes(now);

        let node = match self.nodes.get_mut(nid) {
            Some(node) => node,
            None => return,
        };

        if node.update_block(*block) {
            if block.height > self.best.height {
                self.best = *block;
                log::debug!(
                    "[{}] [nodes={}/feeds={}] new best block={}/{:?}",
                    self.label.0,
                    nodes_len,
                    self.feeds.len(),
                    self.best.height,
                    self.best.hash,
                );
                if let Some(timestamp) = self.timestamp {
                    self.block_times.push(now - timestamp);
                    self.average_block_time = Some(self.block_times.average());
                }
                self.timestamp = Some(now);
                self.serializer.push(feed::BestBlock(
                    self.best.height,
                    now,
                    self.average_block_time,
                ));
                propagation_time = Some(0);
            } else if block.height == self.best.height {
                if let Some(timestamp) = self.timestamp {
                    propagation_time = Some(now - timestamp);
                }
            }

            if let Some(details) = node.update_details(now, propagation_time) {
                self.serializer.push(feed::ImportedBlock(nid, details));
            }
        }
    }
}

impl Handler<UpdateNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: UpdateNode, _: &mut Self::Context) {
        let UpdateNode { nid, payload } = msg;

        if let Some(block) = payload.best_block() {
            self.handle_block(block, nid);
        }

        if let Some(node) = self.nodes.get_mut(nid) {
            match payload {
                Payload::SystemInterval(ref interval) => {
                    if node.update_hardware(interval) {
                        self.serializer.push(feed::Hardware(nid, node.hardware()));
                    }

                    if let Some(stats) = node.update_stats(interval) {
                        self.serializer.push(feed::NodeStatsUpdate(nid, stats));
                    }

                    if let Some(io) = node.update_io(interval) {
                        self.serializer.push(feed::NodeIOUpdate(nid, io));
                    }
                }
                // Payload::AfgAuthoritySet(authority) => {
                //     node.set_validator_address(authority.authority_id.clone());
                //     self.broadcast();
                //     return;
                // }
                // Payload::AfgFinalized(finalized) => {
                //     if let Ok(finalized_number) = finalized.finalized_number.parse::<BlockNumber>()
                //     {
                //         if let Some(addr) = node.details().validator.clone() {
                //             self.serializer.push(feed::AfgFinalized(
                //                 addr,
                //                 finalized_number,
                //                 finalized.finalized_hash,
                //             ));
                //             self.broadcast_finality();
                //         }
                //     }
                //     return;
                // }
                // Payload::AfgReceivedPrecommit(precommit) => {
                //     if let Ok(finalized_number) =
                //         precommit.received.target_number.parse::<BlockNumber>()
                //     {
                //         if let Some(addr) = node.details().validator.clone() {
                //             let voter = precommit.received.voter.clone();
                //             self.serializer.push(feed::AfgReceivedPrecommit(
                //                 addr,
                //                 finalized_number,
                //                 precommit.received.target_hash,
                //                 voter,
                //             ));
                //             self.broadcast_finality();
                //         }
                //     }
                //     return;
                // }
                // Payload::AfgReceivedPrevote(prevote) => {
                //     if let Ok(finalized_number) =
                //         prevote.received.target_number.parse::<BlockNumber>()
                //     {
                //         if let Some(addr) = node.details().validator.clone() {
                //             let voter = prevote.received.voter.clone();
                //             self.serializer.push(feed::AfgReceivedPrevote(
                //                 addr,
                //                 finalized_number,
                //                 prevote.received.target_hash,
                //                 voter,
                //             ));
                //             self.broadcast_finality();
                //         }
                //     }
                //     return;
                // }
                // Payload::AfgReceivedCommit(_) => {}
                _ => (),
            }

            if let Some(block) = payload.finalized_block() {
                if let Some(finalized) = node.update_finalized(block) {
                    self.serializer.push(feed::FinalizedBlock(
                        nid,
                        finalized.height,
                        finalized.hash,
                    ));

                    if finalized.height > self.finalized.height {
                        self.finalized = *finalized;
                        self.serializer
                            .push(feed::BestFinalized(finalized.height, finalized.hash));
                    }
                }
            }
        }

        self.broadcast();
    }
}

impl Handler<LocateNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: LocateNode, _: &mut Self::Context) {
        let LocateNode { nid, location } = msg;

        if let Some(node) = self.nodes.get_mut(nid) {
            self.serializer.push(feed::LocatedNode(
                nid,
                location.latitude,
                location.longitude,
                &location.city,
            ));

            node.update_location(location);
        }
    }
}

impl Handler<RemoveNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: RemoveNode, ctx: &mut Self::Context) {
        let RemoveNode(nid) = msg;

        if let Some(node) = self.nodes.remove(nid) {
            self.decrement_label_count(&node.details().chain);
        }

        if self.nodes.is_empty() {
            log::info!("[{}] Lost all nodes, dropping...", self.label.0);
            ctx.stop();
        }

        self.serializer.push(feed::RemovedNode(nid));
        self.broadcast();
        self.update_count();
    }
}

impl Handler<Subscribe> for Chain {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) {
        let Subscribe(feed) = msg;
        let fid = self.feeds.add(feed.clone());

        feed.do_send(Subscribed(fid, ctx.address().recipient()));

        self.serializer.push(feed::SubscribedTo(&self.label.0));
        self.serializer.push(feed::TimeSync(now()));
        self.serializer.push(feed::BestBlock(
            self.best.height,
            self.timestamp.unwrap_or(0),
            self.average_block_time,
        ));
        self.serializer.push(feed::BestFinalized(
            self.finalized.height,
            self.finalized.hash,
        ));

        for (idx, (nid, node)) in self.nodes.iter().enumerate() {
            // Send subscription confirmation and chain head before doing all the nodes,
            // and continue sending batches of 32 nodes a time over the wire subsequently
            if idx % 32 == 0 {
                if let Some(serialized) = self.serializer.finalize() {
                    feed.do_send(serialized);
                }
            }

            self.serializer.push(feed::AddedNode(nid, node));
            self.serializer.push(feed::FinalizedBlock(
                nid,
                node.finalized().height,
                node.finalized().hash,
            ));
            if node.stale() {
                self.serializer.push(feed::StaleNode(nid));
            }
        }

        if let Some(serialized) = self.serializer.finalize() {
            feed.do_send(serialized);
        }
    }
}

impl Handler<SendFinality> for Chain {
    type Result = ();

    fn handle(&mut self, msg: SendFinality, _ctx: &mut Self::Context) {
        let SendFinality(fid) = msg;
        if let Some(feed) = self.feeds.get(fid) {
            self.finality_feeds.insert(fid, feed.clone());
        }

        // info!("Added new finality feed {}", fid);
    }
}

impl Handler<NoMoreFinality> for Chain {
    type Result = ();

    fn handle(&mut self, msg: NoMoreFinality, _: &mut Self::Context) {
        let NoMoreFinality(fid) = msg;

        // info!("Removed finality feed {}", fid);
        self.finality_feeds.remove(&fid);
    }
}

impl Handler<Unsubscribe> for Chain {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) {
        let Unsubscribe(fid) = msg;

        if let Some(feed) = self.feeds.get(fid) {
            self.serializer.push(feed::UnsubscribedFrom(&self.label.0));

            if let Some(serialized) = self.serializer.finalize() {
                feed.do_send(serialized);
            }
        }

        self.feeds.remove(fid);
        self.finality_feeds.remove(&fid);
    }
}
