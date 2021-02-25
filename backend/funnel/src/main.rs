use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_http::ws::Codec;
use actix_web::{web, get, middleware, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use clap::Clap;
use simple_logger::SimpleLogger;

mod node;
mod types;
mod util;

use node::connector::NodeConnector;
use types::NodeId;
use util::{Locator, LocatorFactory};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const NAME: &'static str = "Substrate Telemetry Funnel";
const ABOUT: &'static str = "This is the Telemetry Funnel that forwards the data sent by Substrate/Polkadot nodes to the aggregator backend";

#[derive(Clap)]
#[clap(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
    #[clap(
        short = 'l',
        long = "listen",
        default_value = "127.0.0.1:8000",
        about = "This is the socket address Telemetry is listening to. This is restricted localhost (127.0.0.1) by default and should be fine for most use cases. If you are using Telemetry in a container, you likely want to set this to '0.0.0.0:8000'"
    )]
    socket: std::net::SocketAddr,
}

/// Entry point for connecting nodes
#[get("/submit")]
async fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    locator: web::Data<Addr<Locator>>,
) -> Result<HttpResponse, Error> {
    let ip = req.connection_info().realip_remote_addr().and_then(|mut addr| {
        if let Some(port_idx) = addr.find(":") {
            addr = &addr[..port_idx];
        }
        addr.parse::<Ipv4Addr>().ok()
    });

    let mut res = ws::handshake(&req)?;
    let locator = locator.get_ref().clone().recipient();

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(locator, ip),
        stream,
        Codec::new().max_size(10 * 1024 * 1024), // 10mb frame limit
    )))
}

/// Telemetry entry point. Listening by default on 127.0.0.1:8000.
/// This can be changed using the `PORT` and `BIND` ENV variables.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().with_level(log::LevelFilter::Info).init().expect("Must be able to start a logger");

    let opts: Opts = Opts::parse();

    let factory = LocatorFactory::new();
    let locator = SyncArbiter::start(4, move || factory.create());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::default())
            .data(locator.clone())
            .service(node_route)
    })
    .bind(opts.socket)?
    .run()
    .await
}
