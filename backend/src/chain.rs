use actix::prelude::*;
use std::sync::Arc;
use bytes::Bytes;
use std::str::FromStr;

use crate::aggregator::{Aggregator, DropChain, NodeCount};
use crate::node::{Node, connector::Initialize, message::{NodeMessage, Details}};
use crate::feed::connector::{FeedId, FeedConnector, Subscribed, Unsubscribed/*, AfgFinalized2*/};
use crate::feed::{self, FeedMessageSerializer};
use crate::util::{DenseMap, NumStats, now};
use crate::types::{NodeId, NodeDetails, NodeLocation, Block, Timestamp, BlockNumber};

const STALE_TIMEOUT: u64 = 2 * 60 * 1000; // 2 minutes

pub type ChainId = usize;
pub type Label = Arc<str>;

pub struct Chain {
    cid: ChainId,
    /// Who to inform if we Chain drops itself
    aggregator: Addr<Aggregator>,
    /// Label of this chain
    label: Label,
    /// Dense mapping of NodeId -> Node
    nodes: DenseMap<Node>,
    /// Dense mapping of FeedId -> Addr<FeedConnector>,
    feeds: DenseMap<Addr<FeedConnector>>,
    /// Dense mapping of FeedId -> Addr<FeedConnector> for feeds requiring finality info,
    // finality_feeds: HashMap<FeedId, Addr<FeedConnector>>,
    finality_feeds: DenseMap<Addr<FeedConnector>>,
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
}

impl Chain {
    pub fn new(cid: ChainId, aggregator: Addr<Aggregator>, label: Label) -> Self {
        info!("[{}] Created", label);

        Chain {
            cid,
            aggregator,
            label,
            nodes: DenseMap::new(),
            feeds: DenseMap::new(),
            finality_feeds: DenseMap::new(),
            best: Block::zero(),
            finalized: Block::zero(),
            block_times: NumStats::new(50),
            average_block_time: None,
            serializer: FeedMessageSerializer::new(),
            timestamp: None,
        }
    }

    fn broadcast(&mut self) {
        if let Some(msg) = self.serializer.finalize() {
            for (_, feed) in self.feeds.iter() {
                feed.do_send(msg.clone());
                info!("BROADCASTING");
            }
        }
    }

    fn broadcast_finality(&mut self) {
        info!("broadcast_finality 1");
        if let Some(msg) = self.serializer.finalize() {
            info!("broadcast_finality 2");
            for (_, feed) in self.finality_feeds.iter() {
                feed.do_send(msg.clone());
                info!("SENDING FINALITY INFO");
            }
        }
    }

    /// Triggered when the number of nodes in this chain has changed, Aggregator will
    /// propagate new counts to all connected feeds
    fn update_count(&self) {
        self.aggregator.do_send(NodeCount(self.cid, self.nodes.len()));
    }

    fn update_average_block_time(&mut self, now: u64) {
        if let Some(timestamp) = self.timestamp {
            self.block_times.push(now - timestamp);
            self.average_block_time = Some(self.block_times.average());
        }
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

        for (nid, node) in self.nodes.iter_mut() {
            if !node.update_stale(threshold) {
                if node.best().height > best.height {
                    best = *node.best();
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
            self.timestamp = None;

            self.serializer.push(feed::BestBlock(self.best.height, now, None));
            self.serializer.push(feed::BestFinalized(finalized.height, finalized.hash));
        }
    }
}

impl Actor for Chain {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        self.aggregator.do_send(DropChain(self.label.clone()));

        for (_, feed) in self.feeds.iter() {
            feed.do_send(Unsubscribed)
        }
    }
}

/// Message sent from the Aggregator to the Chain when new Node is connected
#[derive(Message)]
pub struct AddNode {
    pub node: NodeDetails,
    pub rec: Recipient<Initialize>,
}

/// Message sent from the NodeConnector to the Chain when it receives new telemetry data
#[derive(Message)]
pub struct UpdateNode {
    pub nid: NodeId,
    pub msg: NodeMessage,
    pub raw: Option<Bytes>,
}

/// Message sent from the NodeConnector to the Chain when the connector disconnects
#[derive(Message)]
pub struct RemoveNode(pub NodeId);

/// Message sent from the Aggregator to the Chain when the connector wants to subscribe to that chain
#[derive(Message)]
pub struct Subscribe(pub Addr<FeedConnector>);

/// Message sent from the FeedConnector before it subscribes to a new chain, or if it disconnects
#[derive(Message)]
pub struct Unsubscribe(pub FeedId);

/// TEST TEST TEST
#[derive(Message)]
pub struct SendFinality(pub Addr<FeedConnector>);

#[derive(Message)]
pub struct NoMoreFinality(pub FeedId);

/// Message sent from the NodeConnector to the Chain when it receives location data
#[derive(Message)]
pub struct LocateNode {
    pub nid: NodeId,
    pub location: Arc<NodeLocation>,
}

pub struct GetNodeNetworkState(pub NodeId);

impl Message for GetNodeNetworkState {
    type Result = Option<Bytes>;
}

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let nid = self.nodes.add(Node::new(msg.node));

        if let Err(_) = msg.rec.do_send(Initialize(nid, ctx.address())) {
            self.nodes.remove(nid);
        } else if let Some(node) = self.nodes.get(nid) {
            self.serializer.push(feed::AddedNode(nid, node));
            self.broadcast();
        }

        self.update_count();
    }
}

impl Handler<UpdateNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: UpdateNode, _: &mut Self::Context) {
        let UpdateNode { nid, msg, raw } = msg;

        if let Some(block) = msg.details.best_block() {
            let mut propagation_time = None;
            let now = now();

            self.update_stale_nodes(now);

            if block.height > self.best.height {
                self.best = *block;
                info!(
                    "[{}] [{}/{}] new best block ({}) {:?}",
                    self.label,
                    self.nodes.len(),
                    self.feeds.len(),
                    self.best.height,
                    self.best.hash,
                );
                self.update_average_block_time(now);
                self.timestamp = Some(now);
                self.serializer.push(feed::BestBlock(self.best.height, now, self.average_block_time));
                propagation_time = Some(0);
            } else if block.height == self.best.height {
                if let Some(timestamp) = self.timestamp {
                    propagation_time = Some(now - timestamp);
                }
            }

            if let Some(node) = self.nodes.get_mut(nid) {
                if let Some(details) = node.update_block(*block, now, propagation_time) {
                    self.serializer.push(feed::ImportedBlock(nid, details));
                }
            }
        }

        if let Some(node) = self.nodes.get_mut(nid) {
            match msg.details {
                Details::SystemInterval(ref interval) => {
                    if interval.network_state.is_some() {
                        if let Some(raw) = raw {
                            node.set_network_state(raw);
                        }
                    }

                    if node.update_hardware(interval) {
                        self.serializer.push(feed::Hardware(nid, node.hardware()));
                    }

                    if let Some(stats) = node.update_stats(interval) {
                        self.serializer.push(feed::NodeStatsUpdate(nid, stats));
                    }
                }
                Details::SystemNetworkState(_) => {
                    if let Some(raw) = raw {
                        node.set_network_state(raw);
                    }
                }
                Details::AfgAuthoritySet(authority) => {
                    // info!("Node.address? {}", node.details().name);
                    info!("AfgAuthoritySet>>>>>>>");
                    
                    node.set_validator_address(authority.authority_id.clone());

                    // let addr = authority.authority_id.clone();
                    // info!("AfgAuthoritySet({}, {}, {}, {}, {})", addr, authority.authorities,
                    //     authority.authority_id, self.best.height, self.best.hash);//, number, authority.hash);
                    // self.serializer.push(feed::AfgAuthoritySet(addr, authority.authorities,
                    //     authority.authority_id, self.best.height, self.best.hash));//, number, authority.hash));

                    self.broadcast();
                    info!("AfgAuthoritySet!!!!!!!!");

                    ///////////////////////////////
                    // if let Ok(authority_id) = authority.authority_id.parse::<Address>() {
                    //     if let Ok(number) = authority.number.parse::<BlockNumber>() {
                    // if let Some(address) = node.details.validator.clone()
                    // info!("Node.address? {}", node.details().name);

                    // let addr = authority.authority_id.clone();
                    // // if let Some(addr) = node.details().validator.clone() {

                    //     info!("AfgAuthoritySet({}, {}, {}, {}, {})", addr, authority.authorities,
                    //         authority.authority_id, self.best.height, self.best.hash);//, number, authority.hash);
                    //     self.serializer.push(feed::AfgAuthoritySet(addr, authority.authorities,
                    //         authority.authority_id, self.best.height, self.best.hash));//, number, authority.hash));
                    //     self.broadcast_finality();
                        //     }
                        // }
                    // }
                    ///////////////////////////////
                    return;
                }
                Details::AfgFinalized(ref finalized) => {
                    info!("Reached AfgFinalized");
                    if let Ok(finalized_number) = finalized.finalized_number.parse::<BlockNumber>() {
                        let finalized_hash = finalized.finalized_hash;

                        // let addr = Box::from(nid.to_string());
                        // let addr = node.details().validator.clone().unwrap_or(Box::from(""));
                        if let Some(addr) = node.details().validator.clone() {
                            info!("AfgFinalized({}, {}, {})", addr, finalized_number, finalized_hash);
                            self.serializer.push(feed::AfgFinalized(addr, finalized_number, finalized_hash));
                            self.broadcast_finality();
                        }
                    }
                    return;
                }
                Details::AfgReceivedPrecommit(ref precommit) => {
                    info!("Reached AfgReceivedPrecommit ************");
                    if let Ok(finalized_number) = precommit.received.target_number.parse::<BlockNumber>() {
                        let finalized_hash = precommit.received.target_hash;
                        if let Some(addr) = node.details().validator.clone() {
                            let voter = precommit.received.voter.clone();
                            info!("AfgReceivedPrecommit({}, {}, {}, {})", addr, finalized_number, finalized_hash, voter);
                            self.serializer.push(feed::AfgReceivedPrecommit(addr, finalized_number, finalized_hash, voter));
                            self.broadcast_finality();
                        }
                    }
                    return;
                }
                Details::AfgReceivedPrevote(ref prevote) => {
                    info!("Reached AfgReceivedPrevote ************");
                    if let Ok(finalized_number) = prevote.received.target_number.parse::<BlockNumber>() {
                        let finalized_hash = prevote.received.target_hash;
                        if let Some(addr) = node.details().validator.clone() {
                            let voter = prevote.received.voter.clone();
                            info!("AfgReceivedPrevote({}, {}, {}, {})", addr, finalized_number, finalized_hash, voter);
                            self.serializer.push(feed::AfgReceivedPrevote(addr, finalized_number, finalized_hash, voter));
                            self.broadcast_finality();
                        }
                    }
                    return;
                }
                Details::AfgReceivedCommit(ref commit) => {
                    // info!("Reached AfgReceivedCommit ************");
                }
                _ => (),
            }

            if let Some(block) = msg.details.finalized_block() {
                if let Some(finalized) = node.update_finalized(block) {
                    self.serializer.push(feed::FinalizedBlock(nid, finalized.height, finalized.hash));

                    if finalized.height > self.finalized.height {
                        self.finalized = *finalized;
                        self.serializer.push(feed::BestFinalized(finalized.height, finalized.hash));
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
            self.serializer.push(feed::LocatedNode(nid, location.latitude, location.longitude, &location.city));

            node.update_location(location);
        }
    }
}

impl Handler<RemoveNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: RemoveNode, ctx: &mut Self::Context) {
        let RemoveNode(nid) = msg;

        self.nodes.remove(nid);

        if self.nodes.is_empty() {
            info!("[{}] Lost all nodes, dropping...", self.label);
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

        self.serializer.push(feed::SubscribedTo(&self.label));
        self.serializer.push(feed::TimeSync(now()));
        self.serializer.push(feed::BestBlock(
            self.best.height,
            self.timestamp.unwrap_or_else(|| 0),
            self.average_block_time,
        ));
        self.serializer.push(feed::BestFinalized(self.finalized.height, self.finalized.hash));

        for (nid, node) in self.nodes.iter() {
            self.serializer.push(feed::AddedNode(nid, node));
            self.serializer.push(feed::FinalizedBlock(nid, node.finalized().height, node.finalized().hash));
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

    fn handle(&mut self, msg: SendFinality, ctx: &mut Self::Context) {
        let SendFinality(feed) = msg;
        let fid = self.finality_feeds.add(feed.clone());

        info!("Added new finality feed {}", fid);

    }
}

impl Handler<NoMoreFinality> for Chain {
    type Result = ();

    fn handle(&mut self, msg: NoMoreFinality, _: &mut Self::Context) {
        let NoMoreFinality(fid) = msg;

        // if let Some(feed) = self.finality_feeds.get(fid) {
        //     // self.serializer.push(feed::UnsubscribedFrom(&self.label));

        //     if let Some(serialized) = self.serializer.finalize() {
        //         feed.do_send(serialized);
        //     }
        // }

        info!("Removed finality feed {}", fid);

        self.finality_feeds.remove(fid);
    }
}

impl Handler<Unsubscribe> for Chain {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) {
        let Unsubscribe(fid) = msg;

        info!("Unsubscribe-----");

        if let Some(feed) = self.feeds.get(fid) {
            self.serializer.push(feed::UnsubscribedFrom(&self.label));

            if let Some(serialized) = self.serializer.finalize() {
                feed.do_send(serialized);
            }
        }

        self.feeds.remove(fid);
        self.finality_feeds.remove(fid);
    }
}

impl Handler<GetNodeNetworkState> for Chain {
    type Result = <GetNodeNetworkState as Message>::Result;

    fn handle(&mut self, msg: GetNodeNetworkState, _: &mut Self::Context) -> Self::Result {
        let GetNodeNetworkState(nid) = msg;

        self.nodes.get(nid)?.network_state()
    }
}
