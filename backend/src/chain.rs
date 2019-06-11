use actix::prelude::*;

use crate::aggregator::DropChain;
use crate::node::{Node, NodeId, NodeDetails, connector::Initialize, message::{NodeMessage, Block}};
use crate::feed::connector::{FeedId, FeedConnector};
use crate::util::DenseMap;

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

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let nid = self.nodes.add(Node::new(msg.node));

        if let Err(_) = msg.rec.do_send(Initialize(nid, ctx.address())) {
            self.nodes.remove(nid);
        }
    }
}

impl Handler<UpdateNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: UpdateNode, _: &mut Self::Context) {
        let UpdateNode { nid, msg } = msg;

        if let Some(block) = msg.details.best_block() {
            if block.height > self.best.height {
                self.best = *block;
                info!("[{}] [{}/{}] new best block ({}) {:?}", self.label, self.nodes.len(), self.feeds.len(), self.best.height, self.best.hash);
            }
        }

        if let Some(node) = self.nodes.get_mut(nid) {
            node.update(msg);
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
    }
}

impl Handler<Unsubscribe> for Chain {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, ctx: &mut Self::Context) {
        let Unsubscribe(fid) = msg;

        self.feeds.remove(fid);
    }
}
