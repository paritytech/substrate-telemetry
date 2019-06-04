use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use crate::aggregator::{Aggregator, AddNode};
use crate::chain::{Chain, UpdateNode, RemoveNode};
use crate::node::NodeId;
use crate::node_message::{NodeMessage, Details, SystemConnected};

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
    pub fn new(aggregator: Addr<Aggregator>) -> Self {
        Self {
            // Garbage id, will be replaced by the Initialize message
            nid: !0,
            hb: Instant::now(),
            aggregator,
            chain: None,
            backlog: Vec::new(),
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
        self.chain = Some(chain);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for NodeConnector {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => {
                if let Ok(msg) = serde_json::from_str::<NodeMessage>(&text) {
                    self.handle_message(msg, ctx);
                }
            }
            ws::Message::Close(_) => ctx.stop(),
            _ => (),
        }
    }
}
