use actix::prelude::*;
use actix_web_actors::ws::{CloseCode, CloseReason};
use ctor::ctor;
use std::collections::{HashMap, HashSet};

use crate::chain::{self, Chain, ChainId, Label};
use crate::feed::connector::{Connected, FeedConnector, FeedId};
use crate::feed::{self, FeedMessageSerializer};
use crate::node::connector::{Mute, NodeConnector};
use crate::types::{ConnId, NodeDetails};
use crate::util::{DenseMap, Hash};

pub struct Aggregator {
    genesis_hashes: HashMap<Hash, ChainId>,
    labels: HashMap<Label, ChainId>,
    chains: DenseMap<ChainEntry>,
    feeds: DenseMap<Addr<FeedConnector>>,
    serializer: FeedMessageSerializer,
    /// Denylist for networks we do not want to allow connecting.
    denylist: HashSet<String>,
}

pub struct ChainEntry {
    /// Address to the `Chain` agent
    addr: Addr<Chain>,
    /// Genesis [`Hash`] of the chain
    genesis_hash: Hash,
    /// String name of the chain
    label: Label,
    /// Node count
    nodes: usize,
    /// Maximum allowed nodes
    max_nodes: usize,
}

#[ctor]
/// Labels of chains we consider "first party". These chains allow any
/// number of nodes to connect.
static FIRST_PARTY_NETWORKS: HashSet<&'static str> = {
    let mut set = HashSet::new();
    set.insert("Polkadot");
    set.insert("Kusama");
    set.insert("Westend");
    set.insert("Rococo");
    set
};

/// Max number of nodes allowed to connect to the telemetry server.
const THIRD_PARTY_NETWORKS_MAX_NODES: usize = 500;

impl Aggregator {
    pub fn new(denylist: HashSet<String>) -> Self {
        Aggregator {
            genesis_hashes: HashMap::new(),
            labels: HashMap::new(),
            chains: DenseMap::new(),
            feeds: DenseMap::new(),
            serializer: FeedMessageSerializer::new(),
            denylist,
        }
    }

    /// Get an address to the chain actor by name. If the address is not found,
    /// or the address is disconnected (actor dropped), create a new one.
    pub fn lazy_chain(
        &mut self,
        genesis_hash: Hash,
        label: &str,
        ctx: &mut <Self as Actor>::Context,
    ) -> ChainId {
        let cid = match self.genesis_hashes.get(&genesis_hash).copied() {
            Some(cid) => cid,
            None => {
                self.serializer
                    .push(feed::AddedChain(&label, genesis_hash, 1));

                let addr = ctx.address();
                let max_nodes = max_nodes(label);
                let label: Label = label.into();
                let cid = self.chains.add_with(|cid| ChainEntry {
                    addr: Chain::new(cid, genesis_hash, addr, label.clone()).start(),
                    genesis_hash,
                    label: label.clone(),
                    nodes: 1,
                    max_nodes,
                });

                self.labels.insert(label, cid);
                self.genesis_hashes.insert(genesis_hash, cid);

                self.broadcast();

                cid
            }
        };

        cid
    }

    fn get_chain(&mut self, genesis_hash: &Hash) -> Option<&mut ChainEntry> {
        let chains = &mut self.chains;
        self.genesis_hashes
            .get(genesis_hash)
            .and_then(move |&cid| chains.get_mut(cid))
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
    /// Genesis [`Hash`] of the chain the node is being added to.
    pub genesis_hash: Hash,
    /// Connection id used by the node connector for multiplexing parachains
    pub conn_id: ConnId,
    /// Address of the NodeConnector actor
    pub node_connector: Addr<NodeConnector>,
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
    pub genesis_hash: Hash,
    pub feed: Addr<FeedConnector>,
}

/// Message sent from the FeedConnector to the Aggregator consensus requested
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendFinality {
    pub genesis_hash: Hash,
    pub fid: FeedId,
}

/// Message sent from the FeedConnector to the Aggregator no more consensus required
#[derive(Message)]
#[rtype(result = "()")]
pub struct NoMoreFinality {
    pub genesis_hash: Hash,
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

/// Message sent to the Aggregator to get a health check
#[derive(Message)]
#[rtype(result = "usize")]
pub struct GetHealth;

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        if self.denylist.contains(&*msg.node.chain) {
            log::warn!(target: "Aggregator::AddNode", "'{}' is on the denylist.", msg.node.chain);
            let AddNode { node_connector, .. } = msg;
            let reason = CloseReason {
                code: CloseCode::Abnormal,
                description: Some("Denied".into()),
            };
            node_connector.do_send(Mute { reason });
            return;
        }
        let AddNode {
            node,
            genesis_hash,
            conn_id,
            node_connector,
        } = msg;
        log::trace!(target: "Aggregator::AddNode", "New node connected. Chain '{}'", node.chain);

        let cid = self.lazy_chain(genesis_hash, &node.chain, ctx);
        let chain = self
            .chains
            .get_mut(cid)
            .expect("Entry just created above; qed");
        if chain.nodes < chain.max_nodes {
            chain.addr.do_send(chain::AddNode {
                node,
                conn_id,
                node_connector,
            });
        } else {
            log::warn!(target: "Aggregator::AddNode", "Chain {} is over quota ({})", chain.label, chain.max_nodes);
            let reason = CloseReason {
                code: CloseCode::Again,
                description: Some("Overquota".into()),
            };
            node_connector.do_send(Mute { reason });
        }
    }
}

impl Handler<DropChain> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: DropChain, _: &mut Self::Context) {
        let DropChain(cid) = msg;

        if let Some(entry) = self.chains.remove(cid) {
            let label = &entry.label;
            self.genesis_hashes.remove(&entry.genesis_hash);
            self.labels.remove(label);
            self.serializer.push(feed::RemovedChain(entry.genesis_hash));
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
            self.serializer.push(feed::RemovedChain(entry.genesis_hash));
            self.serializer
                .push(feed::AddedChain(&new, entry.genesis_hash, entry.nodes));

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
        let Subscribe { genesis_hash, feed } = msg;

        if let Some(chain) = self.get_chain(&genesis_hash) {
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
        let SendFinality { genesis_hash, fid } = msg;
        if let Some(chain) = self.get_chain(&genesis_hash) {
            chain.addr.do_send(chain::SendFinality(fid));
        }
    }
}

impl Handler<NoMoreFinality> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: NoMoreFinality, _: &mut Self::Context) {
        let NoMoreFinality { genesis_hash, fid } = msg;
        if let Some(chain) = self.get_chain(&genesis_hash) {
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

        self.serializer.push(feed::Version(32));

        // TODO: keep track on number of nodes connected to each chain
        for (_, entry) in self.chains.iter() {
            self.serializer.push(feed::AddedChain(
                &entry.label,
                entry.genesis_hash,
                entry.nodes,
            ));
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
                self.serializer
                    .push(feed::AddedChain(&entry.label, entry.genesis_hash, count));
                self.broadcast();
            }
        }
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
