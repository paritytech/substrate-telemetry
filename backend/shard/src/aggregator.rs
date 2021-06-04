use std::net::Ipv4Addr;
use std::fmt;
// use std::sync::mpsc::{self, Sender};

use actix::prelude::*;
use actix_http::http::Uri;
use bincode::Options;
use rustc_hash::FxHashMap;
use shared::util::{Hash, DenseMap};
use shared::types::{ConnId, NodeDetails, NodeId};
use shared::node::Payload;
use shared::shard::{ShardConnId, ShardMessage, BackendMessage};
use soketto::handshake::{Client, ServerResponse};
use crate::node::{NodeConnector, Initialize};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

type WsSender = soketto::Sender<Compat<TcpStream>>;
type WsReceiver = soketto::Receiver<Compat<TcpStream>>;

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
        let (tx_ret, mut rx) = mpsc::unbounded_channel();

        let tx = tx_ret.clone();

        tokio::task::spawn(async move {
            let mut sender = match self.connect(tx.clone()).await {
                Ok(pair) => pair,
                Err(err) => {
                    log::error!("Failed to connect to Backend Core: {:?}", err);
                    return;
                }
            };

            // tokio::task::spawn(async move {

            // });

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
                    Some(ChainMessage::UpdateNode(nid, payload)) => {
                        let msg = ShardMessage::UpdateNode {
                            nid,
                            payload,
                        };

                        println!("Serialize {:?}", msg);

                        let bytes = bincode::options().serialize(&msg).unwrap();

                        println!("Sending update: {} bytes", bytes.len());

                        let _ = sender.send_binary_mut(bytes).await;
                        let _ = sender.flush().await;
                    },
                    Some(ChainMessage::Backend(BackendMessage::Initialize { sid, nid })) => {
                        if let Some((addr, conn_id)) = self.nodes.get(sid as usize) {
                            addr.do_send(Initialize {
                                nid,
                                conn_id: *conn_id,
                                chain: tx.clone(),
                            })
                        }
                    },
                    Some(ChainMessage::Backend(BackendMessage::Mute { sid, reason })) => {
                        // TODO
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

        tx_ret
    }

    pub async fn connect(&self, tx: UnboundedSender<ChainMessage>) -> anyhow::Result<WsSender> {
        let host = self.url.host().unwrap_or("127.0.0.1");
        let port = self.url.port_u16().unwrap_or(8000);
        let path = format!("{}{}", self.url.path(), self.genesis_hash);

        let socket = TcpStream::connect((host, port)).await?;

        socket.set_nodelay(true).unwrap();

        let mut client = Client::new(socket.compat(), host, &path);

        let (sender, receiver) = match client.handshake().await? {
            ServerResponse::Accepted { .. } => client.into_builder().finish(),
            ServerResponse::Redirect { status_code, .. } |
            ServerResponse::Rejected { status_code } => {
                return Err(anyhow::anyhow!("Failed to connect, status code: {}", status_code));
            }
        };

        async fn read(tx: UnboundedSender<ChainMessage>, mut receiver: WsReceiver) -> anyhow::Result<()> {
            let mut data = Vec::with_capacity(128);

            loop {
                data.clear();

                receiver.receive_data(&mut data).await?;

                println!("Received {} bytes from Backend Core", data.len());

                match bincode::options().deserialize(&data) {
                    Ok(msg) => tx.send(ChainMessage::Backend(msg))?,
                    Err(err) => {
                        log::error!("Failed to read message from Backend Core: {:?}", err);
                    }
                }

            }
        }

        tokio::task::spawn(read(tx, receiver));

        Ok(sender)
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
    UpdateNode(NodeId, Payload),
    Backend(BackendMessage),
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
