use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use std::net::Ipv4Addr;
use std::mem;

use bytes::{Bytes, BytesMut};
use actix::prelude::*;
use actix_web_actors::ws;
use actix_http::ws::Item;
use crate::aggregator::{Aggregator, AddNode};
use crate::chain::{Chain, UpdateNode, RemoveNode};
use crate::node::NodeId;
use crate::node::message::{NodeMessage, Details};
use crate::util::LocateRequest;
use crate::types::ConnId;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Continuation buffer limit, 10mb
const CONT_BUF_LIMIT: usize = 10 * 1024 * 1024;

pub struct NodeConnector {
    /// Multiplexing connections by id
    multiplex: BTreeMap<ConnId, ConnMultiplex>,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
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
        backlog: Vec<NodeMessage>,
    }
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
    pub fn new(aggregator: Addr<Aggregator>, locator: Recipient<LocateRequest>, ip: Option<Ipv4Addr>) -> Self {
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
                ctx.stop();
            }
        });
    }

    fn handle_message(&mut self, msg: NodeMessage, data: Bytes, ctx: &mut <Self as Actor>::Context) {
        let conn_id = msg.id();

        match self.multiplex.entry(conn_id).or_default() {
            ConnMultiplex::Connected { nid, chain } => {
                chain.do_send(UpdateNode {
                    nid: *nid,
                    msg,
                    raw: Some(data),
                });
            }
            ConnMultiplex::Waiting { backlog } => {
                if let Details::SystemConnected(connected) = msg.details() {
                    let mut node = connected.node.clone();
                    let rec = ctx.address().recipient();

                    // FIXME: Use genesis hash instead of names to avoid this mess
                    match &*node.chain {
                        "Kusama CC3" => node.chain = "Kusama".into(),
                        "Polkadot CC1" => node.chain = "Polkadot".into(),
                        _ => (),
                    }

                    self.aggregator.do_send(AddNode { rec, conn_id, node });
                } else {
                    if backlog.len() >= 10 {
                        backlog.remove(0);
                    }

                    backlog.push(msg);
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
pub struct Initialize {
    pub nid: NodeId,
    pub conn_id: ConnId,
    pub chain: Addr<Chain>,
}

impl Handler<Initialize> for NodeConnector {
    type Result = ();

    fn handle(&mut self, msg: Initialize, _: &mut Self::Context) {
        let Initialize { nid, conn_id, chain } = msg;

        let mx = self.multiplex.entry(conn_id).or_default();

        if let ConnMultiplex::Waiting { backlog } = mx {
            for msg in backlog.drain(..) {
                chain.do_send(UpdateNode { nid, msg, raw: None });
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
            Ok(ws::Message::Text(text)) => text.into(),
            Ok(ws::Message::Binary(data)) => data,
            Ok(ws::Message::Close(_)) => {
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
            }
            Err(error) => {
                log::error!("{:?}", error);
                ctx.stop();
                return;
            }
        };

        match serde_json::from_slice(&data) {
            Ok(msg) => {
                self.handle_message(msg, data, ctx)
            },
            #[cfg(debug)]
            Err(err) => {
                let data: &[u8] = data.get(..512).unwrap_or_else(|| &data);
                warn!("Failed to parse node message: {} {}", err, std::str::from_utf8(data).unwrap_or_else(|_| "INVALID UTF8"))
            },
            #[cfg(not(debug))]
            Err(_) => (),
        }
    }
}
