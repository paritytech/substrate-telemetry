use std::net::{IpAddr, SocketAddr};
use warp::filters::addr;
use warp::filters::header;
use warp::Filter;

/**
A warp filter to extract the "real" IP address of the connection by looking at headers
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

Return `None` if all of this fails to yield an address.
*/
pub fn real_ip() -> impl warp::Filter<Extract = (Option<IpAddr>,), Error = warp::Rejection> + Clone
{
    header::optional("forwarded")
        .and(header::optional("x-forwarded-for"))
        .and(header::optional("x-real-ip"))
        .and(addr::remote())
        .map(pick_best_ip_from_options)
}

fn pick_best_ip_from_options(
    // Forwarded header value (if present)
    forwarded: Option<String>,
    // X-Forwarded-For header value (if present)
    forwarded_for: Option<String>,
    // X-Real-IP header value (if present)
    real_ip: Option<String>,
    // socket address (if known)
    addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    let realip = forwarded
        .as_ref()
        .and_then(|val| get_first_addr_from_forwarded_header(val))
        .or_else(|| {
            // fall back to X-Forwarded-For
            forwarded_for
                .as_ref()
                .and_then(|val| get_first_addr_from_x_forwarded_for_header(val))
        })
        .or_else(|| {
            // fall back to X-Real-IP
            real_ip.as_ref().map(|val| val.trim())
        })
        .and_then(|ip| {
            // Try parsing assuming it may have a port first,
            // and then assuming it doesn't.
            ip.parse::<SocketAddr>()
                .map(|s| s.ip())
                .or_else(|_| ip.parse::<IpAddr>())
                .ok()
        })
        // Fall back to local IP address if the above fails
        .or(addr.map(|a| a.ip()));

    realip
}

/// Follow https://datatracker.ietf.org/doc/html/rfc7239 to decode the Forwarded header value.
/// Roughly, proxies can add new sets of values by appending a comma to the existing list
/// (so we have something like "values1, values2, values3" from proxy1, proxy2 and proxy3 for
/// instance) and then the valeus themselves are ';' separated name=value pairs. The value in each
/// pair may or may not be surrounded in double quotes.
///
/// Examples from the RFC:
///
///   Forwarded: for="_gazonk"
///   Forwarded: For="[2001:db8:cafe::17]:4711"
///   Forwarded: for=192.0.2.60;proto=http;by=203.0.113.43
///   Forwarded: for=192.0.2.43, for=198.51.100.17
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
