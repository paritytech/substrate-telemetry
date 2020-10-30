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
use crate::node::message::{NodeMessage, Details, SystemConnected};
use crate::util::LocateRequest;
use crate::types::ConnId;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Continuation buffer limit, 10mb
const CONT_BUF_LIMIT: usize = 10 * 1024 * 1024;

pub struct NodeConnector {
    /// Id of the node this connector is responsible for handling
    nid: NodeId,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// Mapping message connection id to addresses of chains for multiplexing
    /// a node running multiple parachains
    chains: BTreeMap<ConnId, Addr<Chain>>,
    /// Backlog of messages to be sent once we get a recipient handle to the chain
    backlogs: BTreeMap<ConnId, Vec<NodeMessage>>,
    /// IP address of the node this connector is responsible for
    ip: Option<Ipv4Addr>,
    /// Actix address of location services
    locator: Recipient<LocateRequest>,
    /// Buffer for constructing continuation messages
    contbuf: BytesMut,
}

impl Actor for NodeConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        for chain in self.chains.values() {
            chain.do_send(RemoveNode(self.nid));
        }
    }
}

impl NodeConnector {
    pub fn new(aggregator: Addr<Aggregator>, locator: Recipient<LocateRequest>, ip: Option<Ipv4Addr>) -> Self {
        Self {
            // Garbage id, will be replaced by the Initialize message
            nid: !0,
            hb: Instant::now(),
            aggregator,
            chains: BTreeMap::new(),
            backlogs: BTreeMap::new(),
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
        let conn_id = msg.id.unwrap_or(0);

        if let Some(chain) = self.chains.get(&conn_id) {
            chain.do_send(UpdateNode {
                nid: self.nid,
                msg,
                raw: Some(data)
            });

            return;
        }

        if let Details::SystemConnected(connected) = msg.details {
            let SystemConnected { network_id: _, mut node } = connected;
            let rec = ctx.address().recipient();

            // FIXME: Use genesis hash instead of names to avoid this mess
            match &*node.chain {
                "Kusama CC3" => node.chain = "Kusama".into(),
                "Polkadot CC1" => node.chain = "Polkadot".into(),
                _ => (),
            }

            self.aggregator.do_send(AddNode { rec, conn_id, node });
        } else {
            let backlog = self.backlogs.entry(conn_id).or_default();

            if backlog.len() >= 10 {
                backlog.remove(0);
            }

            backlog.push(msg);
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

        if let Some(backlog) = self.backlogs.remove(&conn_id) {
            for msg in backlog {
                chain.do_send(UpdateNode { nid, msg, raw: None });
            }
        }

        self.nid = nid;
        self.chains.insert(conn_id, chain.clone());

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
                // info!("New node message: {}", std::str::from_utf8(&data).unwrap_or_else(|_| "INVALID UTF8"));
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
