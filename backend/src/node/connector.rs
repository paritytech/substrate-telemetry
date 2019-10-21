use std::time::{Duration, Instant};
use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_web_actors::ws;
use crate::aggregator::{Aggregator, AddNode};
use crate::chain::{Chain, UpdateNode, RemoveNode};
use crate::node::NodeId;
use crate::node::message::{NodeMessage, Details, SystemConnected};
use crate::util::LocateRequest;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct NodeConnector {
    /// Id of the node this connector is responsible for handling
    nid: NodeId,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// Chain actor address
    chain: Option<Addr<Chain>>,
    /// Backlog of messages to be sent once we get a recipient handle to the chain
    backlog: Vec<NodeMessage>,
    /// IP address of the node this connector is responsible for
    ip: Option<Ipv4Addr>,
    /// Actix address of location services
    locator: Recipient<LocateRequest>,
}

impl Actor for NodeConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        if let Some(chain) = self.chain.as_ref() {
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
            chain: None,
            backlog: Vec::new(),
            ip,
            locator,
        }
    }

    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // stop actor
                ctx.stop();
            } else {
                ctx.ping("")
            }
        });
    }

    fn handle_message(&mut self, msg: NodeMessage, ctx: &mut <Self as Actor>::Context) {
        if let Some(chain) = self.chain.as_ref() {
            chain.do_send(UpdateNode {
                nid: self.nid,
                msg,
            });

            return;
        }

        if let Details::SystemConnected(connected) = msg.details {
            let SystemConnected { chain, node } = connected;
            let rec = ctx.address().recipient();
            let chain = chain.into();

            self.aggregator.do_send(AddNode { rec, chain, node });
        } else {
            self.backlog.push(msg);
        }
    }
}

#[derive(Message)]
pub struct Initialize(pub NodeId, pub Addr<Chain>);

impl Handler<Initialize> for NodeConnector {
    type Result = ();

    fn handle(&mut self, msg: Initialize, _: &mut Self::Context) {
        let Initialize(nid, chain) = msg;

        for msg in self.backlog.drain(..) {
            chain.do_send(UpdateNode { nid, msg });
        }

        // At this point backlog should never be used again, so we can free the memory
        self.backlog.shrink_to_fit();

        self.nid = nid;
        self.chain = Some(chain.clone());

        // Acquire the node's physical location
        if let Some(ip) = self.ip {
            println!("wtf {:?}", ip);

            let _ = self.locator.send(LocateRequest { ip, nid, chain });
        }
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for NodeConnector {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        let msg = match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
                None
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
                None
            }
            ws::Message::Text(text) => serde_json::from_str(&text).ok(),
            ws::Message::Binary(data) => serde_json::from_slice(&data).ok(),
            ws::Message::Close(_) => {
                ctx.stop();
                None
            }
            ws::Message::Nop => None,
        };

        if let Some(msg) = msg {
            self.handle_message(msg, ctx);
        }
    }
}
