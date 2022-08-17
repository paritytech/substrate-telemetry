// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use futures::io::{BufReader, BufWriter};
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use std::future::Future;
use std::net::SocketAddr;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

/// A convenience function to start up a Hyper server and handle requests.
pub async fn start_server<H, F>(addr: SocketAddr, handler: H) -> Result<(), anyhow::Error>
where
    H: Clone + Send + Sync + 'static + FnMut(SocketAddr, Request<Body>) -> F,
    F: Send + 'static + Future<Output = Result<Response<Body>, anyhow::Error>>,
{
    let service = hyper::service::make_service_fn(move |addr: &AddrStream| {
        let mut handler = handler.clone();
        let addr = addr.remote_addr();
        async move { Ok::<_, hyper::Error>(hyper::service::service_fn(move |r| handler(addr, r))) }
    });
    let server = Server::bind(&addr).serve(service);

    log::info!("listening on http://{}", server.local_addr());
    server.await?;

    Ok(())
}

type WsStream = BufReader<BufWriter<Compat<hyper::upgrade::Upgraded>>>;
pub type WsSender = soketto::connection::Sender<WsStream>;
pub type WsReceiver = soketto::connection::Receiver<WsStream>;

/// A convenience function to upgrade a Hyper request into a Soketto Websocket.
pub fn upgrade_to_websocket<H, F>(req: Request<Body>, on_upgrade: H) -> hyper::Response<Body>
where
    H: 'static + Send + FnOnce(WsSender, WsReceiver) -> F,
    F: Send + Future<Output = ()>,
{
    if !is_upgrade_request(&req) {
        return basic_response(400, "Expecting WebSocket upgrade headers");
    }

    let key = match req.headers().get("Sec-WebSocket-Key") {
        Some(key) => key,
        None => {
            return basic_response(
                400,
                "Upgrade to websocket connection failed; Sec-WebSocket-Key header not provided",
            )
        }
    };

    if req
        .headers()
        .get("Sec-WebSocket-Version")
        .map(|v| v.as_bytes())
        != Some(b"13")
    {
        return basic_response(
            400,
            "Sec-WebSocket-Version header should have a value of 13",
        );
    }

    // Just a little ceremony to return the correct response key:
    let mut accept_key_buf = [0; 32];
    let accept_key = generate_websocket_accept_key(key.as_bytes(), &mut accept_key_buf);

    // Tell the client that we accept the upgrade-to-WS request:
    let response = Response::builder()
        .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
        .header(hyper::header::CONNECTION, "upgrade")
        .header(hyper::header::UPGRADE, "websocket")
        .header("Sec-WebSocket-Accept", accept_key)
        .body(Body::empty())
        .expect("bug: failed to build response");

    // Spawn our handler to work with the WS connection:
    tokio::spawn(async move {
        // Get our underlying TCP stream:
        let stream = match hyper::upgrade::on(req).await {
            Ok(stream) => stream,
            Err(e) => {
                log::error!("Error upgrading connection to websocket: {}", e);
                return;
            }
        };

        // Start a Soketto server with it:
        let server =
            soketto::handshake::Server::new(BufReader::new(BufWriter::new(stream.compat())));

        // Get hold of a way to send and receive messages:
        let (sender, receiver) = server.into_builder().finish();

        // Pass these to our when-upgraded handler:
        on_upgrade(sender, receiver).await;
    });

    response
}

/// A helper to return a basic HTTP response with a code and text body.
fn basic_response(code: u16, msg: impl AsRef<str>) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::from(msg.as_ref().to_owned()))
        .expect("bug: failed to build response body")
}

/// Defined in RFC 6455. this is how we convert the Sec-WebSocket-Key in a request into a
/// Sec-WebSocket-Accept that we return in the response.
fn generate_websocket_accept_key<'a>(key: &[u8], buf: &'a mut [u8; 32]) -> &'a [u8] {
    // Defined in RFC 6455, we append this to the key to generate the response:
    const KEY: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    use sha1::{Digest, Sha1};
    let mut digest = Sha1::new();
    digest.update(key);
    digest.update(KEY);
    let d = digest.finalize();

    let n = base64::encode_config_slice(&d, base64::STANDARD, buf);
    &buf[..n]
}

/// Check if a request is a websocket upgrade request.
fn is_upgrade_request<B>(request: &hyper::Request<B>) -> bool {
    header_contains_value(request.headers(), hyper::header::CONNECTION, b"upgrade")
        && header_contains_value(request.headers(), hyper::header::UPGRADE, b"websocket")
}

/// Check if there is a header of the given name containing the wanted value.
fn header_contains_value(
    headers: &hyper::HeaderMap,
    header: hyper::header::HeaderName,
    value: &[u8],
) -> bool {
    pub fn trim(x: &[u8]) -> &[u8] {
        let from = match x.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &[],
        };
        let to = x.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &x[from..=to]
    }

    for header in headers.get_all(header) {
        if header
            .as_bytes()
            .split(|&c| c == b',')
            .any(|x| trim(x).eq_ignore_ascii_case(value))
        {
            return true;
        }
    }
    false
}
