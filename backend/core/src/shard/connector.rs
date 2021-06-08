use std::time::{Duration, Instant};
use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use crate::aggregator::{AddNode, Aggregator, NodeSource};
use crate::chain::{Chain, RemoveNode, UpdateNode};
use crate::location::LocateRequest;
use actix::prelude::*;
use actix_web_actors::ws::{self, CloseReason};
use bincode::Options;
use shared::types::NodeId;
use shared::util::Hash;
use shared::ws::{MultipartHandler, WsMessage};
use shared::shard::{ShardMessage, ShardConnId, BackendMessage};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct ShardConnector {
    /// Client must send ping at least once every 60 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// Genesis hash of the chain this connection will be submitting data for
    genesis_hash: Hash,
    /// Chain address to which this shard connector is delegating messages
    chain: Option<Addr<Chain>>,
    /// Transient mapping of `ShardConnId` to external IP address.
    ips: BTreeMap<ShardConnId, Ipv4Addr>,
    /// Mapping of `ShardConnId` to initialized `NodeId`s.
    nodes: BTreeMap<ShardConnId, NodeId>,
    /// Actix address of location services
    locator: Recipient<LocateRequest>,
    /// Container for handling continuation messages
    multipart: MultipartHandler,
}

impl Actor for ShardConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        if let Some(ref chain) = self.chain {
            for nid in self.nodes.values() {
                chain.do_send(RemoveNode(*nid))
            }
        }
    }
}

impl ShardConnector {
    pub fn new(
        aggregator: Addr<Aggregator>,
        locator: Recipient<LocateRequest>,
        genesis_hash: Hash,
    ) -> Self {
        Self {
            hb: Instant::now(),
            aggregator,
            genesis_hash,
            chain: None,
            ips: BTreeMap::new(),
            nodes: BTreeMap::new(),
            locator,
            multipart: MultipartHandler::default(),
        }
    }

    fn shard_send(msg: BackendMessage, ctx: &mut <Self as Actor>::Context) {
        let bytes = bincode::options().serialize(&msg).expect("Must be able to serialize to vec; qed");

        println!("Sending back {} bytes", bytes.len());

        ctx.binary(bytes);
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

    fn handle_message(&mut self, msg: ShardMessage, ctx: &mut <Self as Actor>::Context) {
        println!("{:?}", msg);

        match msg {
            ShardMessage::AddNode { ip, node, sid } => {
                if let Some(ip) = ip {
                    self.ips.insert(sid, ip);
                }

                self.aggregator.do_send(AddNode {
                    node,
                    genesis_hash: self.genesis_hash,
                    source: NodeSource::Shard {
                        sid,
                        shard_connector: ctx.address(),
                    }
                });
            },
            ShardMessage::UpdateNode { nid, payload } => {
                if let Some(chain) = self.chain.as_ref() {
                    chain.do_send(UpdateNode {
                        nid,
                        payload,
                    });
                }
            },
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Initialize {
    pub nid: NodeId,
    pub sid: ShardConnId,
    pub chain: Addr<Chain>,
}

impl Handler<Initialize> for ShardConnector {
    type Result = ();

    fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) {
        let Initialize {
            nid,
            sid,
            chain,
        } = msg;
        log::trace!(target: "ShardConnector::Initialize", "Initializing a node, nid={}, on conn_id={}", nid, 0);

        if self.chain.is_none() {
            self.chain = Some(chain.clone());
        }

        let be_msg = BackendMessage::Initialize { sid, nid };

        Self::shard_send(be_msg, ctx);

        // Acquire the node's physical location
        if let Some(ip) = self.ips.remove(&sid) {
            let _ = self.locator.do_send(LocateRequest { ip, nid, chain });
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ShardConnector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.hb = Instant::now();

        let data = match msg.map(|msg| self.multipart.handle(msg)) {
            Ok(WsMessage::Nop) => return,
            Ok(WsMessage::Ping(msg)) => {
                ctx.pong(&msg);
                return;
            }
            Ok(WsMessage::Data(data)) => data,
            Ok(WsMessage::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            Err(error) => {
                log::error!("{:?}", error);
                ctx.stop();
                return;
            }
        };

        match bincode::options().deserialize(&data) {
            Ok(msg) => self.handle_message(msg, ctx),
            // #[cfg(debug)]
            Err(err) => {
                log::warn!("Failed to parse shard message: {}", err,)
            }
            // #[cfg(not(debug))]
            // Err(_) => (),
        }
    }
}
