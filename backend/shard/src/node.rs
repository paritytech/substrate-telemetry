use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use crate::aggregator::{AddNode, Aggregator, ChainMessage};
// use crate::chain::{Chain, RemoveNode, UpdateNode};
use actix::prelude::*;
use actix_web_actors::ws::{self, CloseReason};
use shared::node::{NodeMessage, Payload};
use shared::types::{ConnId, NodeId};
use shared::ws::{MultipartHandler, WsMessage};
use tokio::sync::mpsc::UnboundedSender;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct NodeConnector {
    /// Multiplexing connections by id
    multiplex: BTreeMap<ConnId, ConnMultiplex>,
    /// Client must send ping at least once every 60 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// IP address of the node this connector is responsible for
    ip: Option<Ipv4Addr>,
    /// Helper for handling continuation messages
    multipart: MultipartHandler,
}

enum ConnMultiplex {
    Connected {
        /// Id of the node this multiplex connector is responsible for handling
        nid: NodeId,
        /// Chain address to which this multiplex connector is delegating messages
        chain: UnboundedSender<ChainMessage>,
    },
    Waiting {
        /// Backlog of messages to be sent once we get a recipient handle to the chain
        backlog: Vec<Payload>,
    },
}

impl Default for ConnMultiplex {
    fn default() -> Self {
        ConnMultiplex::Waiting {
            backlog: Vec::new(),
        }
    }
}

impl Actor for NodeConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        // for mx in self.multiplex.values() {
        //     if let ConnMultiplex::Connected { chain, nid } = mx {
        //         chain.do_send(RemoveNode(*nid));
        //     }
        // }
    }
}

impl NodeConnector {
    pub fn new(aggregator: Addr<Aggregator>, ip: Option<Ipv4Addr>) -> Self {
        Self {
            multiplex: BTreeMap::new(),
            hb: Instant::now(),
            aggregator,
            ip,
            multipart: MultipartHandler::default(),
        }
    }

    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // stop actor
                ctx.close(Some(CloseReason {
                    code: ws::CloseCode::Abnormal,
                    description: Some("Missed heartbeat".into()),
                }));
                ctx.stop();
            }
        });
    }

    fn handle_message(
        &mut self,
        msg: NodeMessage,
        ctx: &mut <Self as Actor>::Context,
    ) {
        let conn_id = msg.id();
        let payload = msg.into();

        match self.multiplex.entry(conn_id).or_default() {
            ConnMultiplex::Connected { nid, chain } => {
                // TODO: error handle
                let _ = chain.send(ChainMessage::UpdateNode(*nid, payload));
            }
            ConnMultiplex::Waiting { backlog } => {
                if let Payload::SystemConnected(connected) = payload {
                    println!("Node connected {:?}", connected.node);
                    self.aggregator.do_send(AddNode {
                        genesis_hash: connected.genesis_hash,
                        ip: self.ip,
                        node: connected.node,
                        conn_id,
                        node_connector: ctx.address(),
                    });
                } else {
                    if backlog.len() >= 10 {
                        backlog.remove(0);
                    }

                    backlog.push(payload);
                }
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Initialize {
    pub nid: NodeId,
    pub conn_id: ConnId,
    pub chain: UnboundedSender<ChainMessage>,
}

impl Handler<Initialize> for NodeConnector {
    type Result = ();

    fn handle(&mut self, msg: Initialize, _: &mut Self::Context) {
        let Initialize {
            nid,
            conn_id,
            chain,
        } = msg;
        log::trace!(target: "NodeConnector::Initialize", "Initializing a node, nid={}, on conn_id={}", nid, conn_id);
        let mx = self.multiplex.entry(conn_id).or_default();

        if let ConnMultiplex::Waiting { backlog } = mx {
            for payload in backlog.drain(..) {
                // TODO: error handle.
                let _ = chain.send(ChainMessage::UpdateNode(nid, payload));
            }

            *mx = ConnMultiplex::Connected {
                nid,
                chain,
            };
        };
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NodeConnector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.hb = Instant::now();

        let data = match msg.map(|msg| self.multipart.handle(msg)) {
            Ok(WsMessage::Nop) => return,
            Ok(WsMessage::Ping(msg)) => {
                ctx.pong(&msg);
                return;
            }
            Ok(WsMessage::Data(data)) => data,
            Ok(WsMessage::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            Err(error) => {
                log::error!("{:?}", error);
                ctx.stop();
                return;
            }
        };

        match serde_json::from_slice(&data) {
            Ok(msg) => self.handle_message(msg, ctx),
            #[cfg(debug)]
            Err(err) => {
                let data: &[u8] = data.get(..512).unwrap_or_else(|| &data);
                log::warn!(
                    "Failed to parse node message: {} {}",
                    err,
                    std::str::from_utf8(data).unwrap_or_else(|_| "INVALID UTF8")
                )
            }
            #[cfg(not(debug))]
            Err(_) => (),
        }
    }
}
