use actix::WeakAddr;
use actix::prelude::*;
use rustc_hash::FxHashMap;

use crate::node::Node;
use crate::node_connector::NodeConnector;
use crate::node_message::SystemConnected;
use crate::chain::{Chain, AddNode};

pub struct Aggregator {
    chains: FxHashMap<Box<str>, Addr<Chain>>,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            chains: FxHashMap::default(),
        }
    }

    /// Get an address to the chain actor by name. If the address is not found,
    /// or the address is disconnected (actor dropped), create a new one.
    pub fn lazy_chain(&mut self, chain: &str) -> &Addr<Chain> {
        let connected = self.chains.get(chain).map(|addr| addr.connected()).unwrap_or(false);

        if !connected {
            self.chains.insert(chain.into(), Chain::new(chain.into()).start());
        }

        self.chains.get(chain).expect("Chain has just been inserted if necessary")
    }
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, _: &mut Context<Self>) {
        let chain = msg.chain.clone();

        self.lazy_chain(&chain).do_send(msg);
    }
}
