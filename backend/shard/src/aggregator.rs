use std::net::Ipv4Addr;
use std::fmt;
// use std::sync::mpsc::{self, Sender};

use actix::prelude::*;
use actix_http::http::Uri;
use bincode::Options;
use rustc_hash::FxHashMap;
use shared::util::{Hash, DenseMap};
use shared::types::{ConnId, NodeDetails};
use shared::shard::{ShardConnId, ShardMessage};
use soketto::handshake::{Client, ServerResponse};
use crate::node::NodeConnector;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

type WsChannel<T> = (soketto::Sender<Compat<T>>, soketto::Receiver<Compat<T>>);

#[derive(Default)]
pub struct Aggregator {
    url: Uri,
    chains: FxHashMap<Hash, UnboundedSender<ChainMessage>>,
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

impl Aggregator {
    pub fn new(url: Uri) -> Self {
        Aggregator {
            url,
            chains: Default::default(),
        }
    }
}

pub struct Chain {
    /// Base URL of Backend Core
    url: Uri,
    /// Genesis hash of the chain, required to construct the URL to connect to the Backend Core
    genesis_hash: Hash,
    /// Dense mapping of SharedConnId -> Addr<NodeConnector> + multiplexing ConnId sent from the node.
    nodes: DenseMap<(Addr<NodeConnector>, ConnId)>,
}

impl Chain {
    pub fn new(url: Uri, genesis_hash: Hash) -> Self {
        Chain {
            url,
            genesis_hash,
            nodes: DenseMap::new(),
        }
    }

    pub fn spawn(mut self) -> UnboundedSender<ChainMessage> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        tokio::task::spawn(async move {
            let (mut sender, mut receiver) = match self.connect().await {
                Ok(pair) => pair,
                Err(err) => {
                    log::error!("Failed to connect to Backend Core: {:?}", err);
                    return;
                }
            };

            loop {
                match rx.recv().await {
                    Some(ChainMessage::AddNode(msg)) => {
                        println!("Add node {:?}", msg);

                        let AddNode { node, ip, conn_id, node_connector, .. } = msg;
                        let sid = self.nodes.add((node_connector, conn_id)) as ShardConnId;

                        let bytes = bincode::options().serialize(&ShardMessage::AddNode {
                            ip,
                            node,
                            sid,
                        }).unwrap();

                        println!("Sending {} bytes", bytes.len());

                        let _ = sender.send_binary_mut(bytes).await;
                        let _ = sender.flush().await;
                    },
                    None => (),
                }
            }
            // let mut client = Client::new(socket.compat(), host, &path);

            // let (mut sender, mut receiver) = match client.handshake().await? {
            //     ServerResponse::Accepted { .. } => client.into_builder().finish(),
            //     ServerResponse::Redirect { status_code, location } => unimplemented!("follow location URL"),
            //     ServerResponse::Rejected { status_code } => unimplemented!("handle failure")
            // };
        });

        tx
    }

    pub async fn connect(&self) -> anyhow::Result<WsChannel<TcpStream>> {
        let host = self.url.host().unwrap_or("127.0.0.1");
        let port = self.url.port_u16().unwrap_or(8000);
        let path = format!("{}{}", self.url.path(), self.genesis_hash);

        println!("Path {}", path);

        let socket = TcpStream::connect((host, port)).await?;

        socket.set_nodelay(true).unwrap();

        let mut client = Client::new(socket.compat(), host, &path);

        match client.handshake().await? {
            ServerResponse::Accepted { .. } => Ok(client.into_builder().finish()),
            ServerResponse::Redirect { status_code, .. } |
            ServerResponse::Rejected { status_code } => {
                Err(anyhow::anyhow!("Failed to connect, status code: {}", status_code))
            }
        }
    }
}

impl Actor for Chain {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddNode {
    pub ip: Option<Ipv4Addr>,
    pub genesis_hash: Hash,
    pub node: NodeDetails,
    pub conn_id: ConnId,
    pub node_connector: Addr<NodeConnector>,
}

#[derive(Debug)]
pub enum ChainMessage {
    AddNode(AddNode),
}

impl fmt::Debug for AddNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("AddNode")
    }
}

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode { genesis_hash, .. } = msg;

        let url = &self.url;
        let chain = self
            .chains
            .entry(genesis_hash)
            .or_insert_with(move || Chain::new(url.clone(), genesis_hash).spawn());

        if let Err(err) = chain.send(ChainMessage::AddNode(msg)) {
            let msg = err.0;
            log::error!("Failed to add node to chain, shutting down chain");
            self.chains.remove(&genesis_hash);
            // TODO: Send a message back to clean up node connections
        }
    }
}

impl Handler<AddNode> for Chain {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode { ip, node_connector, .. } = msg;

        println!("Node connected to {}: {:?}", self.genesis_hash, ip);
    }
}
