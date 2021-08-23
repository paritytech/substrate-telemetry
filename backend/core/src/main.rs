use std::collections::HashSet;
use std::iter::FromIterator;
use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_http::ws::Codec;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use clap::Clap;
use simple_logger::SimpleLogger;

mod aggregator;
mod chain;
mod feed;
mod node;
mod shard;
mod tracker;
mod types;
mod util;

use crate::tracker::Tracker;
use aggregator::{Aggregator, GetHealth};
use feed::connector::FeedConnector;
use node::connector::NodeConnector;
use shard::connector::ShardConnector;
use sqlx::migrate::Migrator;
use sqlx::PgPool;
use util::{Locator, LocatorFactory};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = "Substrate Telemetry Backend";
const ABOUT: &str = "This is the Telemetry Backend that injects and provide the data sent by Substrate/Polkadot nodes";

#[derive(Clap, Debug)]
#[clap(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
    #[clap(
        short = 'l',
        long = "listen",
        default_value = "127.0.0.1:8000",
        about = "This is the socket address Telemetry is listening to. This is restricted to localhost (127.0.0.1) by default and should be fine for most use cases. If you are using Telemetry in a container, you likely want to set this to '0.0.0.0:8000'"
    )]
    socket: std::net::SocketAddr,
    #[clap(
        required = false,
        long = "denylist",
        about = "Space delimited list of chains that are not allowed to connect to telemetry. Case sensitive."
    )]
    denylist: Vec<String>,
    #[clap(
        arg_enum,
        required = false,
        long = "log",
        default_value = "info",
        about = "Log level."
    )]
    log_level: LogLevel,
    #[clap(
        short = 'd',
        long = "db_string",
        about = "An optional postgresql connection string for node uptime tracking, in the following format: postgres://postgres:password@localhost/postgres"
    )]
    db_string: Option<String>,
}

#[derive(Clap, Debug, PartialEq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<&LogLevel> for log::LevelFilter {
    fn from(log_level: &LogLevel) -> Self {
        match log_level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// Entry point for connecting nodes
#[get("/submit/{access_key}")]
async fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    locator: web::Data<Addr<Locator>>,
    tracker: web::Data<Addr<Tracker>>,
    path: web::Path<Box<str>>,
) -> Result<HttpResponse, Error> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .and_then(|mut addr| {
            if let Some(port_idx) = addr.find(':') {
                addr = &addr[..port_idx];
            }
            addr.parse::<Ipv4Addr>().ok()
        });

    let mut res = ws::handshake(&req)?;
    let aggregator = aggregator.get_ref().clone();
    let tracker = tracker.get_ref().clone();
    let locator = locator.get_ref().clone().recipient();

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(aggregator, tracker, locator, ip, path.to_string()),
        stream,
        Codec::new().max_size(10 * 1024 * 1024), // 10mb frame limit
    )))
}

#[get("/shard_submit/{chain_hash}")]
async fn shard_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    path: web::Path<Box<str>>,
) -> Result<HttpResponse, Error> {
    let hash_str = path.into_inner();
    let genesis_hash = hash_str.parse()?;

    let mut res = ws::handshake(&req)?;

    let aggregator = aggregator.get_ref().clone();

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        ShardConnector::new(aggregator, genesis_hash),
        stream,
        Codec::new().max_size(10 * 1024 * 1024), // 10mb frame limit
    )))
}

/// Entry point for connecting feeds
#[get("/feed")]
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

/// Entry point for health check monitoring bots
#[get("/health")]
async fn health(aggregator: web::Data<Addr<Aggregator>>) -> Result<HttpResponse, Error> {
    match aggregator.send(GetHealth).await {
        Ok(count) => {
            let body = format!("Connected chains: {}", count);

            HttpResponse::Ok().body(body).await
        }
        Err(error) => {
            log::error!("Health check mailbox error: {:?}", error);

            HttpResponse::InternalServerError().await
        }
    }
}

static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

/// Telemetry entry point. Listening by default on 127.0.0.1:8000.
/// This can be changed using the `PORT` and `BIND` ENV variables.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();
    let log_level = &opts.log_level;
    let mut pool: Option<PgPool> = None;
    if let Some(db_str) = opts.db_string {
        pool = Some(PgPool::connect(&db_str).await.unwrap());
        MIGRATOR.run(pool.as_ref().unwrap()).await.unwrap();
    }

    SimpleLogger::new()
        .with_level(log_level.into())
        .init()
        .expect("Must be able to start a logger");

    let denylist = HashSet::from_iter(opts.denylist);
    let aggregator = Aggregator::new(denylist).start();
    let tracker = Tracker::new(pool).start();
    let factory = LocatorFactory::new();
    let locator = SyncArbiter::start(4, move || factory.create());
    log::info!("Starting telemetry version: {}", env!("CARGO_PKG_VERSION"));
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::default())
            .data(aggregator.clone())
            .data(locator.clone())
            .data(tracker.clone())
            .service(node_route)
            .service(feed_route)
            .service(health)
    })
    .bind(opts.socket)?
    .run()
    .await
}
