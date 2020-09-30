use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_http::ws::Codec;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use clap::Clap;
use simple_logger::SimpleLogger;

mod aggregator;
mod chain;
mod feed;
mod node;
mod types;
mod util;

use aggregator::{Aggregator, GetHealth, GetNetworkState};
use feed::connector::FeedConnector;
use node::connector::NodeConnector;
use types::NodeId;
use util::{Locator, LocatorFactory};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const NAME: &'static str = "Substrate Telemetry Backend";
const ABOUT: &'static str = "This is the Telemetry Backend that injects and provide the data sent by Substrate/Polkadot nodes";

#[derive(Clap)]
#[clap(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
    #[clap(
        short = "l",
        long = "listen",
        default_value = "127.0.0.1:8000",
        about = "This is the socket address Telemetry is listening to. This is restricted localhost (127.0.0.1) by default and should be fine for most use cases. If you are using Telemetry in a container, you likely want to set this to '0.0.0.0:8000'"
    )]
    socket: std::net::SocketAddr,
}

/// Entry point for connecting nodes
async fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    locator: web::Data<Addr<Locator>>,
) -> Result<HttpResponse, Error> {
    let ip = req.connection_info().realip_remote_addr().and_then(|mut addr| {
        if let Some(port_idx) = addr.find(":") {
            addr = &addr[..port_idx];
        }
        addr.parse::<Ipv4Addr>().ok()
    });

    let mut res = ws::handshake(&req)?;
    let aggregator = aggregator.get_ref().clone();
    let locator = locator.get_ref().clone().recipient();

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(aggregator, locator, ip),
        stream,
        Codec::new().max_size(10 * 1024 * 1024), // 10mb frame limit
    )))
}

/// Entry point for connecting feeds
async fn feed_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        FeedConnector::new(aggregator.get_ref().clone()),
        &req,
        stream,
    )
}

/// Entry point for network state dump
async fn state_route(
    path: web::Path<(Box<str>, NodeId)>,
    aggregator: web::Data<Addr<Aggregator>>,
) -> Result<HttpResponse, Error> {
    let (chain, nid) = path.into_inner();

    let res = match aggregator.send(GetNetworkState(chain, nid)).await {
        Ok(Some(res)) => res.await,
        Ok(None) => Ok(None),
        Err(error) => Err(error)
    };

    match res {
        Ok(Some(body)) => {
            HttpResponse::Ok().content_type("application/json").body(body).await
        },
        Ok(None) => {
            HttpResponse::Ok().body("Node has disconnected or has not submitted its network state yet").await
        },
        Err(error) => {
            log::error!("Network state mailbox error: {:?}", error);

            HttpResponse::InternalServerError().await
        }
    }
}

/// Entry point for health check monitoring bots
async fn health(aggregator: web::Data<Addr<Aggregator>>) -> Result<HttpResponse, Error> {
    match aggregator.send(GetHealth).await {
        Ok(count) => {
            let body = format!("Connected chains: {}", count);

            HttpResponse::Ok().body(body).await
        },
        Err(error) => {
            log::error!("Health check mailbox error: {:?}", error);

            HttpResponse::InternalServerError().await
        }
    }
}

/// Telemetry entry point. Listening by default on 127.0.0.1:8000.
/// This can be changed using the `PORT` and `BIND` ENV variables.
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use web::{get, resource};

    SimpleLogger::new().with_level(log::LevelFilter::Info).init().expect("Must be able to start a logger");

    let opts: Opts = Opts::parse();
    let aggregator = Aggregator::new().start();
    let factory = LocatorFactory::new();
    let locator = SyncArbiter::start(4, move || factory.create());

    HttpServer::new(move || {
        App::new()
            .data(aggregator.clone())
            .data(locator.clone())
            .service(resource("/submit").route(get().to(node_route)))
            .service(resource("/submit/").route(get().to(node_route)))
            .service(resource("/feed").route(get().to(feed_route)))
            .service(resource("/feed/").route(get().to(feed_route)))
            .service(resource("/network_state/{chain}/{nid}").route(get().to(state_route)))
            .service(resource("/network_state/{chain}/{nid}/").route(get().to(state_route)))
            .service(resource("/health").route(get().to(health)))
            .service(resource("/health/").route(get().to(health)))
    })
    .bind(format!("{}", opts.socket))?
    .run()
    .await
}
