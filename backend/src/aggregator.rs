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
    networks: HashMap<Label, ChainId>,
    chains: DenseMap<ChainEntry>,
    feeds: DenseMap<Addr<FeedConnector>>,
    serializer: FeedMessageSerializer,
}

pub struct ChainEntry {
    addr: Addr<Chain>,
    label: Label,
    network_id: Option<Label>,
    nodes: usize,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            labels: HashMap::new(),
            networks: HashMap::new(),
            chains: DenseMap::new(),
            feeds: DenseMap::new(),
            serializer: FeedMessageSerializer::new(),
        }
    }

    /// Get an address to the chain actor by name. If the address is not found,
    /// or the address is disconnected (actor dropped), create a new one.
    pub fn lazy_chain(
        &mut self,
        label: &str,
        network: &Option<Label>,
        ctx: &mut <Self as Actor>::Context,
    ) -> ChainId {
        let cid = match self.get_chain_id(label, network.as_ref()) {
            Some(cid) => cid,
            None => {
                self.serializer.push(feed::AddedChain(&label, 1));

                let addr = ctx.address();
                let label: Label = label.into();
                let cid = self.chains.add_with(|cid| {
                    ChainEntry {
                        addr: Chain::new(cid, addr, label.clone()).start(),
                        label: label.clone(),
                        network_id: network.clone(),
                        nodes: 1,
                    }
                });

                self.labels.insert(label, cid);

                if let Some(network) = network {
                    self.networks.insert(network.clone(), cid);
                }

                self.broadcast();

                cid
            }
        };

        cid
    }

    fn get_chain_id(&self, label: &str, network: Option<&Label>) -> Option<ChainId> {
        let labels = &self.labels;
        let networks = &self.networks;

        if let Some(network) = network {
            networks.get(&**network).or_else(|| labels.get(label)).copied()
        } else {
            labels.get(label).copied()
        }
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
#[rtype(result = "()")]
pub struct AddNode {
    pub node: NodeDetails,
    pub network_id: Option<Label>,
    pub rec: Recipient<Initialize>,
}

/// Message sent from the Chain to the Aggregator when the Chain loses all nodes
#[derive(Message)]
#[rtype(result = "()")]
pub struct DropChain(pub ChainId);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RenameChain(pub ChainId, pub Label);

/// Message sent from the FeedConnector to the Aggregator when subscribing to a new chain
#[derive(Message)]
#[rtype(result = "bool")]
pub struct Subscribe {
    pub chain: Label,
    pub feed: Addr<FeedConnector>,
}

/// Message sent from the FeedConnector to the Aggregator consensus requested
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendFinality {
    pub chain: Label,
    pub fid: FeedId,
}

/// Message sent from the FeedConnector to the Aggregator no more consensus required
#[derive(Message)]
#[rtype(result = "()")]
pub struct NoMoreFinality {
    pub chain: Label,
    pub fid: FeedId,
}

/// Message sent from the FeedConnector to the Aggregator when first connected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect(pub Addr<FeedConnector>);

/// Message sent from the FeedConnector to the Aggregator when disconnecting
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect(pub FeedId);

/// Message sent from the Chain to the Aggergator when the node count on the chain changes
#[derive(Message)]
#[rtype(result = "()")]
pub struct NodeCount(pub ChainId, pub usize);

/// Message sent to the Aggregator to get the network state of a particular node
#[derive(Message)]
#[rtype(result = "Option<Request<Chain, GetNodeNetworkState>>")]
pub struct GetNetworkState(pub Box<str>, pub NodeId);

/// Message sent to the Aggregator to get a health check
#[derive(Message)]
#[rtype(result = "usize")]
pub struct GetHealth;

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode { node, network_id, rec } = msg;

        let cid = self.lazy_chain(&node.chain, &network_id, ctx);
        let chain = self.chains.get_mut(cid).expect("Entry just created above; qed");

        if let Some(network_id) = network_id {
            // Attach network id to the chain if it was not done yet
            if chain.network_id.is_none() {
                chain.network_id = Some(network_id.clone());
                self.networks.insert(network_id, cid);
            }
        }

        chain.addr.do_send(chain::AddNode {
            node,
            rec,
        });
    }
}

impl Handler<DropChain> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: DropChain, _: &mut Self::Context) {
        let DropChain(cid) = msg;

        if let Some(entry) = self.chains.remove(cid) {
            let label = &entry.label;
            self.labels.remove(label);
            if let Some(network) = entry.network_id {
                self.networks.remove(&network);
            }

            self.serializer.push(feed::RemovedChain(label));
            log::info!("Dropped chain [{}] from the aggregator", label);
            self.broadcast();
        }

    }
}

impl Handler<RenameChain> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: RenameChain, _: &mut Self::Context) {
        let RenameChain(cid, new) = msg;

        if let Some(entry) = self.chains.get_mut(cid) {
            if entry.label == new {
                return;
            }

            // Update UI
            self.serializer.push(feed::RemovedChain(&entry.label));
            self.serializer.push(feed::AddedChain(&new, entry.nodes));

            // Update labels -> cid map
            self.labels.remove(&entry.label);
            self.labels.insert(new.clone(), cid);

            // Update entry
            entry.label = new;

            self.broadcast();
        }
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

        log::info!("Feed #{} connected", fid);

        connector.do_send(Connected(fid));

        self.serializer.push(feed::Version(30));

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

        log::info!("Feed #{} disconnected", fid);

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

impl Handler<GetHealth> for Aggregator {
    type Result = usize;

    fn handle(&mut self, _: GetHealth, _: &mut Self::Context) -> Self::Result {
        self.chains.len()
    }
}
