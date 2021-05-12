use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_http::ws::Codec;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use clap::Clap;
use simple_logger::SimpleLogger;

mod node;

use node::connector::NodeConnector;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = "Substrate Telemetry Backend Shard";
const ABOUT: &str = "This is the Telemetry Backend Shard that forwards the data sent by Substrate/Polkadot nodes to the Backend Core";

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
        arg_enum,
        required = false,
        long = "log",
        default_value = "info",
        about = "Log level."
    )]
    log_level: LogLevel,
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
#[get("/submit")]
async fn node_route(
    req: HttpRequest,
    stream: web::Payload,
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

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(ip),
        stream,
        Codec::new().max_size(10 * 1024 * 1024), // 10mb frame limit
    )))
}

/// Telemetry entry point. Listening by default on 127.0.0.1:8000.
/// This can be changed using the `PORT` and `BIND` ENV variables.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let log_level = &opts.log_level;
    SimpleLogger::new()
        .with_level(log_level.into())
        .init()
        .expect("Must be able to start a logger");

    log::info!("Starting Telemetry Shard version: {}", env!("CARGO_PKG_VERSION"));
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::default())
            .service(node_route)
    })
    .bind(opts.socket)?
    .run()
    .await
}
