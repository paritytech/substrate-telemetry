use actix::WeakAddr;
use actix::prelude::*;
use rustc_hash::FxHashMap;

use crate::node::Node;
use crate::node_connector::NodeConnector;
use crate::node_message::SystemConnected;
use crate::chain::Chain;

pub struct Aggregator {
    chains: FxHashMap<Box<str>, Addr<Chain>>,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            chains: FxHashMap::default(),
        }
    }
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

#[derive(Message)]
pub struct AddNode {
    pub connector: WeakAddr<NodeConnector>,
    pub chain: Box<str>,
    pub node: Node,
}

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, _: &mut Context<Self>) {
        let AddNode { connector, chain, node } = msg;

        self.chains
            .entry(chain.clone())
            .or_insert_with(move || Chain::new(chain).start())
            .do_send(crate::chain::AddNode { connector, node });
    }
}
