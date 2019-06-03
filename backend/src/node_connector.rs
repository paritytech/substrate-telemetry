use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use crate::chain::Chain;
use crate::node_message::NodeMessage;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


pub struct NodeConnector {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Chain actor address
    addr: Addr<Chain>,
}

impl Actor for NodeConnector {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl NodeConnector {
    pub fn new(addr: Addr<Chain>) -> Self {
        Self {
            hb: Instant::now(),
            addr,
        }
    }

    /// Send ping every 5 seconds
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
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
                match serde_json::from_str::<NodeMessage>(&text) {
                    Ok(msg) => println!("GOT\t{:?}\nFROM:\t{}\n", msg, text),
                    _ => (),
                    // Err(err) => println!("\t{:?}\n\t{}", err, text),
                }
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
