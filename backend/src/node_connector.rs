use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use crate::aggregator::{Aggregator, AddNode};
use crate::chain::Chain;
use crate::node::Node;
use crate::node_message::{NodeMessage, Details, SystemConnected};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);


pub struct NodeConnector {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// Chain actor address
    chain: Option<Addr<Chain>>,
}

impl Actor for NodeConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

impl NodeConnector {
    pub fn new(aggregator: Addr<Aggregator>) -> Self {
        Self {
            hb: Instant::now(),
            aggregator,
            chain: None,
        }
    }

    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("NodeConnector timeout!");
                // stop actor
                ctx.stop();
            } else {
                ctx.ping("")
            }
        });
    }

    fn with_message(&mut self, msg: NodeMessage, ctx: &mut <Self as Actor>::Context) {
        match msg.details {
            Details::SystemConnected(connected) => {
                let SystemConnected { chain, node } = connected;
                let connector = ctx.address().downgrade();

                self.aggregator.do_send(AddNode { connector, chain, node });
            }
            _ => (), // println!("Unhandled message: {:?}", msg),
        }
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for NodeConnector {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                if let Ok(msg) = serde_json::from_str::<NodeMessage>(&text) {
                    self.with_message(msg, ctx);
                }

                // match serde_json::from_str::<NodeMessage>(&text) {
                //     Ok(msg) => println!("GOT\t{:?}\nFROM:\t{}\n", msg, text),
                //     _ => (),
                //     // Err(err) => println!("\t{:?}\n\t{}", err, text),
                // }
                // ctx.text(test); // echo
            }
            ws::Message::Binary(bin) => {
                println!("Binary message: {} bytes", bin.len());
                // ctx.binary(bin); // echo
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
