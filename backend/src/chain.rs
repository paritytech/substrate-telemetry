use serde::{Serialize, Deserialize};
use actix::WeakAddr;
use actix::prelude::*;
use actix::dev::MessageResponse;

use crate::node::{Node, NodeDetails};
use crate::node_connector::NodeConnector;

#[derive(Serialize, Deserialize, Message, Clone, Copy, Debug)]
pub struct NodeId(usize);

pub struct Chain {
    /// Label of this chain
    label: Box<str>,
    /// List of retired `NodeId`s that can be re-used
    retired: Vec<NodeId>,
    /// All nodes, mapping NodeId.0 -> Node
    nodes: Vec<Option<Node>>,
}

impl Chain {
    pub fn new(label: Box<str>) -> Self {
        println!("New chain created: {}", label);

        Chain {
            label,
            retired: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn add(&mut self, node: Node) -> NodeId {
        match self.retired.pop() {
            Some(nid) => {
                self.nodes[nid.0] = Some(node);
                nid
            },
            None => {
                let nid = NodeId(self.nodes.len());
                self.nodes.push(Some(node));
                nid
            },
        }
    }

    pub fn get(&mut self, nid: NodeId) -> Option<&Node> {
        self.nodes.get(nid.0).and_then(|node| node.as_ref())
    }

    pub fn get_mut(&mut self, nid: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(nid.0).and_then(|node| node.as_mut())
    }

    pub fn remove(&mut self, nid: NodeId) {
        if self.nodes.get_mut(nid.0).take().is_some() {
            self.retired.push(nid);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &Node)> + '_ {
        self.nodes.iter().enumerate().filter_map(|(idx, node)| {
            Some((NodeId(idx), node.as_ref()?))
        })
    }
}

#[derive(Message)]
pub struct AddNode {
    pub node: NodeDetails,
    pub chain: Box<str>,
    pub connector: WeakAddr<NodeConnector>,
}

impl Actor for Chain {
    type Context = Context<Self>;
}

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Context<Self>) {
        println!("[{}] new node {}", self.label, msg.node.name);

        self.add(Node::new(msg.node));
    }
}
