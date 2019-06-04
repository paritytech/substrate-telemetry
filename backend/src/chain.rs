use actix::prelude::*;

use crate::aggregator::DropChain;
use crate::node::{Node, NodeId, NodeDetails};
use crate::node_connector::Initialize;
use crate::node_message::{NodeMessage, Block};
use crate::util::DenseMap;

pub struct Chain {
    /// Who to inform if we Chain drops itself
    drop_rec: Recipient<DropChain>,
    /// Label of this chain
    label: Box<str>,
    /// Dense mapping of NodeId -> Node
    nodes: DenseMap<Node>,
    /// Best block
    best: Block,
}

impl Chain {
    pub fn new(drop_rec: Recipient<DropChain>, label: Box<str>) -> Self {
        info!("[{}] Created", label);

        Chain {
            drop_rec,
            label,
            nodes: DenseMap::new(),
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

#[derive(Message)]
pub struct AddNode {
    pub node: NodeDetails,
    pub rec: Recipient<Initialize>,
}

#[derive(Message)]
pub struct UpdateNode {
    pub nid: NodeId,
    pub msg: NodeMessage,
}

#[derive(Message)]
pub struct RemoveNode(pub NodeId);

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
                info!("[{}] [{}] new best block ({}) {:?}", self.label, self.nodes.len(), self.best.height, self.best.hash);
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
