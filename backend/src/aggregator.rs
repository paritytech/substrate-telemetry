use std::collections::{HashMap, HashSet};
use actix::prelude::*;
use lazy_static::lazy_static;

use crate::node::connector::Initialize;
use crate::feed::connector::{FeedConnector, Connected, FeedId};
use crate::util::DenseMap;
use crate::feed::{self, FeedMessageSerializer};
use crate::chain::{self, Chain, ChainId, Label, GetNodeNetworkState};
use crate::types::{ConnId, NodeDetails, NodeId};

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
    /// Node count
    nodes: usize,
    /// Maximum allowed nodes
    max_nodes: usize,
}

lazy_static! {
    /// Labels of chains we consider "first party". These chains are allowed any
    /// number of nodes to connect.
    static ref FIRST_PARTY_NETWORKS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("Polkadot");
        set.insert("Kusama");
        set.insert("Westend");
        set.insert("Rococo");
        set
    };
}
/// Max number of nodes allowed to connect to the telemetry server.
const THIRD_PARTY_NETWORKS_MAX_NODES: usize = 500;

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
        ctx: &mut <Self as Actor>::Context,
    ) -> ChainId {
        let cid = match self.get_chain_id(label, None.as_ref()) {
            Some(cid) => cid,
            None => {
                self.serializer.push(feed::AddedChain(&label, 1));

                let addr = ctx.address();
                let max_nodes = max_nodes(label);
                let label: Label = label.into();
                let cid = self.chains.add_with(|cid| {
                    ChainEntry {
                        addr: Chain::new(cid, addr, label.clone()).start(),
                        label: label.clone(),
                        network_id: None, // TODO: this doesn't seem to be used anywhere. Can it be removed?
                        nodes: 1,
                        max_nodes,
                    }
                });

                self.labels.insert(label, cid);

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
    /// Details of the node being added to the aggregator
    pub node: NodeDetails,
    /// Connection id used by the node connector for multiplexing parachains
    pub conn_id: ConnId,
    /// Recipient for the initialization message
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
        let AddNode { node, conn_id, rec } = msg;
        log::trace!(target: "Aggregator::AddNode", "New node connected. Chain '{}'", node.chain);

        let cid = self.lazy_chain(&node.chain, ctx);
        let chain = self.chains.get_mut(cid).expect("Entry just created above; qed");
        if chain.nodes < chain.max_nodes {
            chain.addr.do_send(chain::AddNode {
                node,
                conn_id,
                rec,
            });
        } else {
            log::warn!(target: "Aggregator::AddNode", "Chain {} is over quota ({})", chain.label, chain.max_nodes);
        }
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

        self.serializer.push(feed::Version(31));

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

/// First party networks (Polkadot, Kusama etc) are allowed any number of nodes.
/// Third party networks are allowed `THIRD_PARTY_NETWORKS_MAX_NODES` nodes and
/// no more.
fn max_nodes(label: &str) -> usize {
    if FIRST_PARTY_NETWORKS.contains(label) {
        usize::MAX
    } else {
        THIRD_PARTY_NETWORKS_MAX_NODES
    }
}
