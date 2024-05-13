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

use std::net::{IpAddr, SocketAddr};

/**
Extract the "real" IP address of the connection by looking at headers
set by proxies (this is inspired by Actix Web's implementation of the feature).

First, check for the standardised "Forwarded" header. This looks something like:

"Forwarded: for=12.34.56.78;host=example.com;proto=https, for=23.45.67.89"

Each proxy can append to this comma separated list of forwarded-details. We'll look for
the first "for" address and try to decode that.

If this doesn't yield a result, look for the non-standard but common X-Forwarded-For header,
which contains a comma separated list of addresses; each proxy in the potential chain possibly
appending one to the end. So, take the first of these if it exists.

If still no luck, look for the X-Real-IP header, which we expect to contain a single IP address.

If that _still_ doesn't work, fall back to the socket address of the connection.
*/
pub fn real_ip(addr: SocketAddr, headers: &hyper::HeaderMap) -> (IpAddr, Source) {
    let forwarded = headers.get("forwarded").and_then(header_as_str);
    let forwarded_for = headers.get("x-forwarded-for").and_then(header_as_str);
    let real_ip = headers.get("x-real-ip").and_then(header_as_str);
    pick_best_ip_from_options(forwarded, forwarded_for, real_ip, addr)
}

/// The source of the address returned
pub enum Source {
    ForwardedHeader,
    XForwardedForHeader,
    XRealIpHeader,
    SocketAddr,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::ForwardedHeader => write!(f, "'Forwarded' header"),
            Source::XForwardedForHeader => write!(f, "'X-Forwarded-For' header"),
            Source::XRealIpHeader => write!(f, "'X-Real-Ip' header"),
            Source::SocketAddr => write!(f, "Socket address"),
        }
    }
}

fn header_as_str(value: &hyper::header::HeaderValue) -> Option<&str> {
    std::str::from_utf8(value.as_bytes()).ok()
}

fn pick_best_ip_from_options(
    // Forwarded header value (if present)
    forwarded: Option<&str>,
    // X-Forwarded-For header value (if present)
    forwarded_for: Option<&str>,
    // X-Real-IP header value (if present)
    real_ip: Option<&str>,
    // socket address (if known)
    addr: SocketAddr,
) -> (IpAddr, Source) {
    let realip = forwarded
        .as_ref()
        .and_then(|val| {
            let addr = get_first_addr_from_forwarded_header(val)?;
            Some((addr, Source::ForwardedHeader))
        })
        .or_else(|| {
            // fall back to X-Forwarded-For
            forwarded_for.as_ref().and_then(|val| {
                let addr = get_first_addr_from_x_forwarded_for_header(val)?;
                Some((addr, Source::XForwardedForHeader))
            })
        })
        .or_else(|| {
            // fall back to X-Real-IP
            real_ip.as_ref().and_then(|val| {
                let addr = val.trim();
                Some((addr, Source::XRealIpHeader))
            })
        })
        .and_then(|(ip, source)| {
            // Try parsing assuming it may have a port first,
            // and then assuming it doesn't.
            let addr = ip
                .parse::<SocketAddr>()
                .map(|s| s.ip())
                .or_else(|_| ip.parse::<IpAddr>())
                .ok()?;
            Some((addr, source))
        })
        // Fall back to local IP address if the above fails
        .unwrap_or((addr.ip(), Source::SocketAddr));

    realip
}

/// Follow <https://datatracker.ietf.org/doc/html/rfc7239> to decode the Forwarded header value.
/// Roughly, proxies can add new sets of values by appending a comma to the existing list
/// (so we have something like "values1, values2, values3" from proxy1, proxy2 and proxy3 for
/// instance) and then the values themselves are ';' separated name=value pairs. The value in each
/// pair may or may not be surrounded in double quotes.
///
/// Examples from the RFC:
///
/// ```text
/// Forwarded: for="_gazonk"
/// Forwarded: For="[2001:db8:cafe::17]:4711"
/// Forwarded: for=192.0.2.60;proto=http;by=203.0.113.43
/// Forwarded: for=192.0.2.43, for=198.51.100.17
/// ```
fn get_first_addr_from_forwarded_header(value: &str) -> Option<&str> {
    let first_values = value.split(',').next()?;

    for pair in first_values.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let key = parts.next()?;
        let value = parts.next()?;

        if key.to_lowercase() == "for" {
            // trim double quotes if they surround the value:
            let value = if value.starts_with('"') && value.ends_with('"') {
                &value[1..value.len() - 1]
            } else {
                value
            };
            return Some(value);
        }
    }

    None
}

fn get_first_addr_from_x_forwarded_for_header(value: &str) -> Option<&str> {
    value.split(",").map(|val| val.trim()).next()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_addr_from_forwarded_rfc_examples() {
        let examples = vec![
            (r#"for="_gazonk""#, "_gazonk"),
            (
                r#"For="[2001:db8:cafe::17]:4711""#,
                "[2001:db8:cafe::17]:4711",
            ),
            (r#"for=192.0.2.60;proto=http;by=203.0.113.43"#, "192.0.2.60"),
            (r#"for=192.0.2.43, for=198.51.100.17"#, "192.0.2.43"),
        ];

        for (value, expected) in examples {
            assert_eq!(
                get_first_addr_from_forwarded_header(value),
                Some(expected),
                "Header value: {}",
                value
            );
        }
    }
}
