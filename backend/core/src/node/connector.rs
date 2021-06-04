use std::collections::BTreeMap;
use std::mem;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use crate::aggregator::{AddNode, Aggregator};
use crate::chain::{Chain, RemoveNode, UpdateNode};
use crate::node::message::{NodeMessage, Payload};
use crate::node::NodeId;
use crate::types::ConnId;
use crate::util::LocateRequest;
use actix::prelude::*;
use actix_http::ws::Item;
use actix_web_actors::ws::{self, CloseReason};
use bytes::{Bytes, BytesMut};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Continuation buffer limit, 10mb
const CONT_BUF_LIMIT: usize = 10 * 1024 * 1024;

pub struct NodeConnector {
    /// Multiplexing connections by id
    multiplex: BTreeMap<ConnId, ConnMultiplex>,
    /// Client must send ping at least once every 60 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// IP address of the node this connector is responsible for
    ip: Option<Ipv4Addr>,
    /// Actix address of location services
    locator: Recipient<LocateRequest>,
    /// Buffer for constructing continuation messages
    contbuf: BytesMut,
}

enum ConnMultiplex {
    Connected {
        /// Id of the node this multiplex connector is responsible for handling
        nid: NodeId,
        /// Chain address to which this multiplex connector is delegating messages
        chain: Addr<Chain>,
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
        for mx in self.multiplex.values() {
            if let ConnMultiplex::Connected { chain, nid } = mx {
                chain.do_send(RemoveNode(*nid));
            }
        }
    }
}

impl NodeConnector {
    pub fn new(
        aggregator: Addr<Aggregator>,
        locator: Recipient<LocateRequest>,
        ip: Option<Ipv4Addr>,
    ) -> Self {
        Self {
            multiplex: BTreeMap::new(),
            hb: Instant::now(),
            aggregator,
            ip,
            locator,
            contbuf: BytesMut::new(),
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
                chain.do_send(UpdateNode {
                    nid: *nid,
                    payload,
                });
            }
            ConnMultiplex::Waiting { backlog } => {
                if let Payload::SystemConnected(connected) = payload {
                    self.aggregator.do_send(AddNode {
                        node: connected.node,
                        genesis_hash: connected.genesis_hash,
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

    fn start_frame(&mut self, bytes: &[u8]) {
        if !self.contbuf.is_empty() {
            log::error!("Unused continuation buffer");
            self.contbuf.clear();
        }
        self.continue_frame(bytes);
    }

    fn continue_frame(&mut self, bytes: &[u8]) {
        if self.contbuf.len() + bytes.len() <= CONT_BUF_LIMIT {
            self.contbuf.extend_from_slice(&bytes);
        } else {
            log::error!("Continuation buffer overflow");
            self.contbuf = BytesMut::new();
        }
    }

    fn finish_frame(&mut self) -> Bytes {
        mem::replace(&mut self.contbuf, BytesMut::new()).freeze()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Mute {
    pub reason: CloseReason,
}

impl Handler<Mute> for NodeConnector {
    type Result = ();
    fn handle(&mut self, msg: Mute, ctx: &mut Self::Context) {
        let Mute { reason } = msg;
        log::debug!(target: "NodeConnector::Mute", "Muting a node. Reason: {:?}", reason.description);

        ctx.close(Some(reason));
        ctx.stop();
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Initialize {
    pub nid: NodeId,
    pub conn_id: ConnId,
    pub chain: Addr<Chain>,
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
                chain.do_send(UpdateNode {
                    nid,
                    payload,
                });
            }

            *mx = ConnMultiplex::Connected {
                nid,
                chain: chain.clone(),
            };
        };

        // Acquire the node's physical location
        if let Some(ip) = self.ip {
            let _ = self.locator.do_send(LocateRequest { ip, nid, chain });
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NodeConnector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.hb = Instant::now();

        let data = match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                return;
            }
            Ok(ws::Message::Pong(_)) => return,
            Ok(ws::Message::Text(text)) => text.into_bytes(),
            Ok(ws::Message::Binary(data)) => data,
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            Ok(ws::Message::Nop) => return,
            Ok(ws::Message::Continuation(cont)) => match cont {
                Item::FirstText(bytes) | Item::FirstBinary(bytes) => {
                    self.start_frame(&bytes);
                    return;
                }
                Item::Continue(bytes) => {
                    self.continue_frame(&bytes);
                    return;
                }
                Item::Last(bytes) => {
                    self.continue_frame(&bytes);
                    self.finish_frame()
                }
            },
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
