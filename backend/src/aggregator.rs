use std::collections::HashMap;
use actix::prelude::*;

use crate::node::connector::Initialize;
use crate::feed::connector::{FeedConnector, Connected, FeedId};
use crate::util::DenseMap;
use crate::feed::{self, FeedMessageSerializer};
use crate::chain::{self, Chain, ChainId, Label, GetNodeNetworkState};
use crate::types::{NodeDetails, NodeId};

pub struct Aggregator {
    labels: HashMap<Label, ChainId>,
    chains: DenseMap<ChainEntry>,
    feeds: DenseMap<Addr<FeedConnector>>,
    serializer: FeedMessageSerializer,
}

pub struct ChainEntry {
    addr: Addr<Chain>,
    label: Label,
    nodes: usize,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            labels: HashMap::new(),
            chains: DenseMap::new(),
            feeds: DenseMap::new(),
            serializer: FeedMessageSerializer::new(),
        }
    }

    /// Get an address to the chain actor by name. If the address is not found,
    /// or the address is disconnected (actor dropped), create a new one.
    pub fn lazy_chain(&mut self, label: Label, ctx: &mut <Self as Actor>::Context) -> &mut ChainEntry {
        let (cid, found) = self.labels
            .get(&label)
            .map(|&cid| (cid, true))
            .unwrap_or_else(|| {
                self.serializer.push(feed::AddedChain(&label, 1));

                let addr = ctx.address();
                let label = label.clone();
                let cid = self.chains.add_with(move |cid| {
                    ChainEntry {
                        addr: Chain::new(cid, addr, label.clone()).start(),
                        label,
                        nodes: 1,
                    }
                });

                self.broadcast();

                (cid, false)
            });

        if !found {
            self.labels.insert(label, cid);
        }

        self.chains.get_mut(cid).expect("Entry just created above; qed")
    }

    fn get_chain(&mut self, label: &str) -> Option<&mut ChainEntry> {
        let chains = &mut self.chains;
        self.labels.get(label).and_then(move |&cid| chains.get_mut(cid))
    }

    fn broadcast(&mut self) {
        if let Some(msg) = self.serializer.finalize() {
            for (_, feed) in self.feeds.iter() {
                feed.do_send(msg.clone());
            }
        }
    }
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

/// Message sent from the NodeConnector to the Aggregator upon getting all node details
#[derive(Message)]
pub struct AddNode {
    pub node: NodeDetails,
    pub chain: Label,
    pub rec: Recipient<Initialize>,
}

/// Message sent from the Chain to the Aggregator when the Chain loses all nodes
#[derive(Message)]
pub struct DropChain(pub Label);

/// Message sent from the FeedConnector to the Aggregator when subscribing to a new chain
pub struct Subscribe {
    pub chain: Label,
    pub feed: Addr<FeedConnector>,
}

impl Message for Subscribe {
    type Result = bool;
}

/// Message sent from the FeedConnector to the Aggregator consensus requested
#[derive(Message)]
pub struct SendFinality {
    pub chain: Label,
    pub fid: FeedId,
}

/// Message sent from the FeedConnector to the Aggregator no more consensus required
#[derive(Message)]
pub struct NoMoreFinality {
    pub chain: Label,
    pub fid: FeedId,
}

/// Message sent from the FeedConnector to the Aggregator when first connected
#[derive(Message)]
pub struct Connect(pub Addr<FeedConnector>);

/// Message sent from the FeedConnector to the Aggregator when disconnecting
#[derive(Message)]
pub struct Disconnect(pub FeedId);

/// Message sent from the Chain to the Aggergator when the node count on the chain changes
#[derive(Message)]
pub struct NodeCount(pub ChainId, pub usize);

/// Message sent to the Aggregator to get the network state of a particular node
pub struct GetNetworkState(pub Box<str>, pub NodeId);

impl Message for GetNetworkState {
    type Result = Option<Request<Chain, GetNodeNetworkState>>;
}

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode { node, chain, rec } = msg;

        self.lazy_chain(chain, ctx).addr.do_send(chain::AddNode {
            node,
            rec,
        });
    }
}

impl Handler<DropChain> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: DropChain, _: &mut Self::Context) {
        let DropChain(label) = msg;

        if let Some(cid) = self.labels.remove(&label) {
            self.chains.remove(cid);
            self.serializer.push(feed::RemovedChain(&label));
            self.broadcast();
        }

        info!("Dropped chain [{}] from the aggregator", label);
    }
}

impl Handler<Subscribe> for Aggregator {
    type Result = bool;

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> bool {
        let Subscribe { chain, feed } = msg;

        if let Some(chain) = self.get_chain(&chain) {
            chain.addr.do_send(chain::Subscribe(feed));
            true
        } else {
            false
        }
    }
}

impl Handler<SendFinality> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: SendFinality, _: &mut Self::Context) {
        let SendFinality { chain, fid } = msg;
        if let Some(chain) = self.get_chain(&chain) {
            chain.addr.do_send(chain::SendFinality(fid));
        }
    }
}

impl Handler<NoMoreFinality> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: NoMoreFinality, _: &mut Self::Context) {
        let NoMoreFinality { chain, fid } = msg;
        if let Some(chain) = self.get_chain(&chain) {
            chain.addr.do_send(chain::NoMoreFinality(fid));
        }
    }
}

impl Handler<Connect> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        let Connect(connector) = msg;

        let fid = self.feeds.add(connector.clone());

        info!("Feed #{} connected", fid);

        connector.do_send(Connected(fid));

        self.serializer.push(feed::Version(28));

        // TODO: keep track on number of nodes connected to each chain
        for (_, entry) in self.chains.iter() {
            self.serializer.push(feed::AddedChain(&entry.label, entry.nodes));
        }

        if let Some(msg) = self.serializer.finalize() {
            connector.do_send(msg);
        }
    }
}

impl Handler<Disconnect> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        let Disconnect(fid) = msg;

        info!("Feed #{} disconnected", fid);

        self.feeds.remove(fid);
    }
}

impl Handler<NodeCount> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: NodeCount, _: &mut Self::Context) {
        let NodeCount(cid, count) = msg;

        if let Some(entry) = self.chains.get_mut(cid) {
            entry.nodes = count;

            if count != 0 {
                self.serializer.push(feed::AddedChain(&entry.label, count));
                self.broadcast();
            }
        }
    }
}

impl Handler<GetNetworkState> for Aggregator {
    type Result = <GetNetworkState as Message>::Result;

    fn handle(&mut self, msg: GetNetworkState, _: &mut Self::Context) -> Self::Result {
        let GetNetworkState(chain, nid) = msg;

        Some(self.get_chain(&*chain)?.addr.send(GetNodeNetworkState(nid)))
    }
}
