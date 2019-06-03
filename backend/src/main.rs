use actix::prelude::*;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error};
use actix_web_actors::ws;

mod node_connector;
mod node_message;
mod chain;

use node_connector::NodeConnector;
use chain::Chain;

/// Entry point for connecting nodes
fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    chain: web::Data<Addr<Chain>>,
) -> Result<HttpResponse, Error> {
    println!("Connection!");

    ws::start(
        NodeConnector::new(chain.get_ref().clone()),
        &req,
        stream,
    )
}

fn main() -> std::io::Result<()> {
    let sys = System::new("substrate-telemetry");
    let chain = Chain.start();

    HttpServer::new(move || {
        App::new()
            .data(chain.clone())
            .service(web::resource("/submit").route(web::get().to(node_route)))
    })
    .bind("127.0.0.1:8080")?
    .start();

    sys.run()
}
