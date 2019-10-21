use std::time::{Duration, Instant};
use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_web_actors::ws;
use crate::aggregator::{Aggregator, AddNode};
use crate::chain::{Chain, UpdateNode, RemoveNode, LocateNode};
use crate::node::NodeId;
use crate::node::message::{NodeMessage, Details, SystemConnected};
use crate::util::{Post, Location};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Localhost IPv4
pub const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

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
    locator: Addr<crate::util::Locator>,
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
    pub fn new(aggregator: Addr<Aggregator>, locator: Addr<crate::util::Locator>, ip: Option<Ipv4Addr>) -> Self {
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

            self.aggregator.do_send(AddNode { rec, chain, node });
        } else {
            self.backlog.push(msg);
        }
    }

    fn update_node_location(&mut self, location: Location) {
        if let Some(chain) = self.chain.as_ref() {
            chain.do_send(LocateNode {
                nid: self.nid,
                location,
            });
        } else {
            warn!("No chain to send location data to.");
        }
    }
}

#[derive(Message)]
pub struct Initialize(pub NodeId, pub Addr<Chain>);

impl Handler<Initialize> for NodeConnector {
    type Result = ();

    fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) {
        let Initialize(nid, chain) = msg;

        for msg in self.backlog.drain(..) {
            chain.do_send(UpdateNode { nid, msg });
        }

        // At this point backlog should never be used again, so we can free the memory
        self.backlog.shrink_to_fit();

        self.nid = nid;
        self.chain = Some(chain);

        // Acquire the node's physical location
        if let Some(ip) = self.ip {
            if ip == LOCALHOST {
                self.update_node_location(
                    Location { latitude: 52.5166667, longitude: 13.4, city: "Berlin".into() }
                )
            } else {
                self.locator.send(Post { ip })
                .into_actor(self)
                .then(move |res, act, _| {
                    let result = match res {
                        Ok(res) => res,
                        _ => {
                            warn!("Location request unsuccessful");
                            return fut::ok(())
                        }
                    };
                    if let Some(location) = result {
                        act.update_node_location(location);
                    }

                    fut::ok(())
                })
                .wait(ctx);
            }
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
