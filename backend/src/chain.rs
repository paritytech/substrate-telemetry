use actix::prelude::*;

use crate::aggregator::DropChain;
use crate::node::{Node, connector::Initialize, message::{NodeMessage, Block}};
use crate::feed::connector::{FeedId, FeedConnector, Subscribed};
use crate::feed::{self, FeedMessageSerializer, AddedNode, RemovedNode, SubscribedTo, UnsubscribedFrom};
use crate::util::{DenseMap, Location, now};
use crate::types::{NodeId, NodeDetails};

pub type ChainId = usize;

pub struct Chain {
    cid: ChainId,
    /// Who to inform if we Chain drops itself
    drop_rec: Recipient<DropChain>,
    /// Label of this chain
    label: Box<str>,
    /// Dense mapping of NodeId -> Node
    nodes: DenseMap<Node>,
    /// Dense mapping of FeedId -> Addr<FeedConnector>,
    feeds: DenseMap<Addr<FeedConnector>>,
    /// Best block
    best: Block,
    /// Message serializer
    serializer: FeedMessageSerializer,
    /// When the best block first arrived
    timestamp: u64,
}

impl Chain {
    pub fn new(cid: ChainId, drop_rec: Recipient<DropChain>, label: Box<str>) -> Self {
        info!("[{}] Created", label);

        Chain {
            cid,
            drop_rec,
            label,
            nodes: DenseMap::new(),
            feeds: DenseMap::new(),
            best: Block::zero(),
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
}

impl Actor for Chain {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        let _ = self.drop_rec.do_send(DropChain(self.label.clone()));
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
    pub location: Location,
}

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let nid = self.nodes.add(Node::new(msg.node));

        if let Err(_) = msg.rec.do_send(Initialize(nid, ctx.address())) {
            self.nodes.remove(nid);
        } else if let Some(node) = self.nodes.get(nid) {
            self.serializer.push(AddedNode(nid, node.details(), node.stats(),
                node.hardware(), &node.block_details(), node.location()));
            self.broadcast();
        }
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
                info!("[{}] [{}/{}] new best block ({}) {:?}", self.label, self.nodes.len(), self.feeds.len(), self.best.height, self.best.hash);

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
                self.serializer.push(feed::ImportedBlock(nid, &node.block_details()));
            }
        }

        if let Some(node) = self.nodes.get_mut(nid) {
            node.update(msg);
            self.serializer.push(feed::Hardware(nid, node.hardware()));
        }

        self.broadcast();
    }
}

impl Handler<LocateNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: LocateNode, _: &mut Self::Context) {
        if let Some(node) = self.nodes.get_mut(msg.nid) {
            node.update_location(&msg.location);
            let location = node.location();
            self.serializer.push(feed::LocatedNode(msg.nid, location.0, location.1, &location.2));
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

        self.serializer.push(RemovedNode(nid));
        self.broadcast();
    }
}

impl Handler<Subscribe> for Chain {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) {
        let Subscribe(feed) = msg;

        let fid = self.feeds.add(feed.clone());

        feed.do_send(Subscribed(fid, ctx.address().recipient()));

        self.serializer.push(SubscribedTo(&self.label));

        for (nid, node) in self.nodes.iter() {
            self.serializer.push(AddedNode(nid, node.details(), node.stats(),
                node.hardware(), &node.block_details(), node.location()));
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
            self.serializer.push(UnsubscribedFrom(&self.label));

            if let Some(serialized) = self.serializer.finalize() {
                feed.do_send(serialized);
            }
        }

        self.feeds.remove(fid);
    }
}
