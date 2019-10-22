use actix::prelude::*;
use std::sync::Arc;

use crate::aggregator::{Aggregator, DropChain, NodeCount};
use crate::node::{Node, connector::Initialize, message::{NodeMessage, Details}};
use crate::feed::connector::{FeedId, FeedConnector, Subscribed, Unsubscribed};
use crate::feed::{self, FeedMessageSerializer};
use crate::util::{DenseMap, now};
use crate::types::{NodeId, NodeDetails, NodeLocation, Block};

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
    /// Best block
    best: Block,
    /// Finalized block
    finalized: Block,
    /// Message serializer
    serializer: FeedMessageSerializer,
    /// When the best block first arrived
    timestamp: u64,
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
            best: Block::zero(),
            finalized: Block::zero(),
            serializer: FeedMessageSerializer::new(),
            timestamp: now(),
        }
    }

    fn broadcast(&mut self) {
        if let Some(msg) = self.serializer.finalize() {
            for (_, feed) in self.feeds.iter() {
                feed.do_send(msg.clone());
            }
        }
    }

    /// Triggered when the number of nodes in this chain has changed, Aggregator will
    /// propagate new counts to all connected feeds
    fn update_count(&self) {
        self.aggregator.do_send(NodeCount(self.cid, self.nodes.len()));
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

/// Message sent from the NodeConnector to the Chain when it receives location data
#[derive(Message)]
pub struct LocateNode {
    pub nid: NodeId,
    pub location: NodeLocation,
}

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let nid = self.nodes.add(Node::new(msg.node));

        if let Err(_) = msg.rec.do_send(Initialize(nid, ctx.address())) {
            self.nodes.remove(nid);
        } else if let Some(node) = self.nodes.get(nid) {
            self.serializer.push(feed::AddedNode(nid, node.details(), node.stats(),
                node.hardware(), node.block_details(), node.location()));
            self.broadcast();
        }

        self.update_count();
    }
}

impl Handler<UpdateNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: UpdateNode, _: &mut Self::Context) {
        let UpdateNode { nid, msg } = msg;

        if let Some(block) = msg.details.best_block() {
            let mut propagation_time = 0;
            let time_now = now();

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
                self.serializer.push(feed::BestBlock(self.best.height, msg.ts, None));
                self.timestamp = time_now;
            } else if block.height == self.best.height {
                if let Some(node) = self.nodes.get(nid) {
                    if block.height > node.best().height {
                        propagation_time = time_now - self.timestamp;
                    }
                }
            }

            if let Some(node) = self.nodes.get_mut(nid) {
                node.update_block(*block, time_now, propagation_time);
                self.serializer.push(feed::ImportedBlock(nid, node.block_details()));
            }
        }

        if let Some(node) = self.nodes.get_mut(nid) {
            if let Details::SystemInterval(ref interval) = msg.details {
                if node.update_hardware(interval) {
                    self.serializer.push(feed::Hardware(nid, node.hardware()));
                }

                if let Some(finalized) = node.update_finalized(interval) {
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
        // self.serializer.push(feed::BestBlock());
        self.serializer.push(feed::BestFinalized(self.finalized.height, self.finalized.hash));

        for (nid, node) in self.nodes.iter() {
            self.serializer.push(feed::AddedNode(nid, node.details(), node.stats(),
                node.hardware(), node.block_details(), node.location()));
            self.serializer.push(feed::FinalizedBlock(nid, node.finalized().height, node.finalized().hash));
        }

        if let Some(serialized) = self.serializer.finalize() {
            feed.do_send(serialized);
        }
    }
}

impl Handler<Unsubscribe> for Chain {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) {
        let Unsubscribe(fid) = msg;

        if let Some(feed) = self.feeds.get(fid) {
            self.serializer.push(feed::UnsubscribedFrom(&self.label));

            if let Some(serialized) = self.serializer.finalize() {
                feed.do_send(serialized);
            }
        }

        self.feeds.remove(fid);
    }
}
