use crate::aggregator::{Aggregator, Connect, Disconnect, NoMoreFinality, SendFinality, Subscribe};
use crate::chain::Unsubscribe;
use crate::feed::{FeedMessageSerializer, Pong};
use actix::prelude::*;
use actix_web_actors::ws;
use bytes::Bytes;
use shared::util::fnv;
use std::time::{Duration, Instant};

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
    /// FNV hash of the chain label, optimization to avoid double-subscribing
    chain_label_hash: u64,
    /// Message serializer
    serializer: FeedMessageSerializer,
}

impl Actor for FeedConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        self.aggregator.do_send(Connect(ctx.address()));
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        if let Some(chain) = self.chain.take() {
            let _ = chain.do_send(Unsubscribe(self.fid_chain));
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
            chain_label_hash: 0,
            serializer: FeedMessageSerializer::new(),
        }
    }

    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // stop actor
                ctx.stop();
            } else {
                ctx.ping(b"")
            }
        });
    }

    fn handle_cmd(&mut self, cmd: &str, payload: &str, ctx: &mut <Self as Actor>::Context) {
        match cmd {
            "subscribe" => {
                match fnv(payload) {
                    hash if hash == self.chain_label_hash => return,
                    hash => self.chain_label_hash = hash,
                }

                self.aggregator
                    .send(Subscribe {
                        chain: payload.into(),
                        feed: ctx.address(),
                    })
                    .into_actor(self)
                    .then(|res, actor, _| {
                        match res {
                            Ok(true) => (),
                            // Chain not found, reset hash
                            _ => actor.chain_label_hash = 0,
                        }
                        async {}.into_actor(actor)
                    })
                    .wait(ctx);
            }
            "send-finality" => {
                self.aggregator.do_send(SendFinality {
                    chain: payload.into(),
                    fid: self.fid_chain,
                });
            }
            "no-more-finality" => {
                self.aggregator.do_send(NoMoreFinality {
                    chain: payload.into(),
                    fid: self.fid_chain,
                });
            }
            "ping" => {
                self.serializer.push(Pong(payload));
                if let Some(serialized) = self.serializer.finalize() {
                    ctx.binary(serialized.0);
                }
            }
            _ => (),
        }
    }
}

/// Message sent form Chain to the FeedConnector upon successful subscription
#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribed(pub FeedId, pub Recipient<Unsubscribe>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribed;

/// Message sent from Aggregator to FeedConnector upon successful connection
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connected(pub FeedId);

/// Message sent from either Aggregator or Chain to FeedConnector containing
/// serialized message(s) for the frontend
///
/// Since Bytes is ARC'ed, this is cheap to clone
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Serialized(pub Bytes);

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for FeedConnector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => self.hb = Instant::now(),
            Ok(ws::Message::Text(text)) => {
                if let Some(idx) = text.find(':') {
                    let cmd = &text[..idx];
                    let payload = &text[idx + 1..];

                    log::info!("New FEED message: {}", cmd);

                    self.handle_cmd(cmd, payload, ctx);
                }
            }
            Ok(ws::Message::Close(_)) => ctx.stop(),
            Ok(_) => (),
            Err(error) => {
                log::error!("{:?}", error);
                ctx.stop();
            }
        }
    }
}

impl Handler<Subscribed> for FeedConnector {
    type Result = ();

    fn handle(&mut self, msg: Subscribed, _: &mut Self::Context) {
        let Subscribed(fid_chain, chain) = msg;

        if let Some(current) = self.chain.take() {
            let _ = current.do_send(Unsubscribe(self.fid_chain));
        }

        self.fid_chain = fid_chain;
        self.chain = Some(chain);
    }
}

impl Handler<Unsubscribed> for FeedConnector {
    type Result = ();

    fn handle(&mut self, _: Unsubscribed, _: &mut Self::Context) {
        self.chain = None;
        self.chain_label_hash = 0;
    }
}

impl Handler<Connected> for FeedConnector {
    type Result = ();

    fn handle(&mut self, msg: Connected, _: &mut Self::Context) {
        let Connected(fid_aggregator) = msg;

        self.fid_aggregator = fid_aggregator;
    }
}

impl Handler<Serialized> for FeedConnector {
    type Result = ();

    fn handle(&mut self, msg: Serialized, ctx: &mut Self::Context) {
        let Serialized(bytes) = msg;

        ctx.binary(bytes);
    }
}
