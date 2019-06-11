use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use crate::aggregator::{Aggregator, Connect, Disconnect};
use crate::chain::{Chain, Unsubscribe};

pub type FeedId = usize;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct FeedConnector {
    /// FeedId that Aggregator holds of this actor
    fid_aggregator: FeedId,
    /// FeedId that Chain holds of this actor
    fid_chain: FeedId,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// Chain actor address
    chain: Option<Recipient<Unsubscribe>>,
}

impl Actor for FeedConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        self.aggregator.do_send(Connect(ctx.address()));
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        if let Some(chain) = self.chain.as_ref() {
            chain.do_send(Unsubscribe(self.fid_chain));
        }

        self.aggregator.do_send(Disconnect(self.fid_aggregator));
    }
}

impl FeedConnector {
    pub fn new(aggregator: Addr<Aggregator>) -> Self {
        Self {
            // Garbage id, will be replaced by the Connected message
            fid_aggregator: !0,
            // Garbage id, will be replaced by the Subscribed message
            fid_chain: !0,
            hb: Instant::now(),
            aggregator,
            chain: None,
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
}

/// Message sent form Chain to the FeedConnector upon successful subscription
#[derive(Message)]
pub struct Subscribed(pub FeedId, Recipient<Unsubscribe>);

/// Message sent from Aggregator to FeedConnector upon successful connection
#[derive(Message)]
pub struct Connected(pub FeedId);

impl StreamHandler<ws::Message, ws::ProtocolError> for FeedConnector {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => {
                info!("FeedConnector sent: {}", text);
            }
            ws::Message::Close(_) => ctx.stop(),
            _ => (),
        }
    }
}

impl Handler<Subscribed> for FeedConnector {
    type Result = ();

    fn handle(&mut self, msg: Subscribed, ctx: &mut Self::Context) {
        let Subscribed(fid_chain, chain) = msg;

        self.fid_chain = fid_chain;
        self.chain = Some(chain);
    }
}

impl Handler<Connected> for FeedConnector {
    type Result = ();

    fn handle(&mut self, msg: Connected, ctx: &mut Self::Context) {
        let Connected(fid_aggregator) = msg;

        self.fid_aggregator = fid_aggregator;
    }
}
