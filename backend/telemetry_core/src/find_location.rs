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

use std::net::Ipv4Addr;
use std::sync::Arc;

use futures::{Sink, SinkExt};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use anyhow::Context;
use common::node_types::NodeLocation;
use tokio::sync::Semaphore;

/// The returned location is optional; it may be None if not found.
pub type Location = Option<Arc<NodeLocation>>;

/// This is responsible for taking an IP address and attempting
/// to find a geographical location from this
pub fn find_location<Id, R>(response_chan: R) -> flume::Sender<(Id, Ipv4Addr)>
where
    R: Sink<(Id, Option<Arc<NodeLocation>>)> + Unpin + Send + Clone + 'static,
    Id: Clone + Send + 'static,
{
    let (tx, rx) = flume::unbounded();

    // cache entries
    let mut cache: FxHashMap<Ipv4Addr, Arc<NodeLocation>> = FxHashMap::default();

    // Default entry for localhost
    cache.insert(
        Ipv4Addr::new(127, 0, 0, 1),
        Arc::new(NodeLocation {
            latitude: 52.516_6667,
            longitude: 13.4,
            city: "Berlin".into(),
        }),
    );

    // Create a locator with our cache. This is used to obtain locations.
    let locator = Locator::new(cache);

    // Spawn a loop to handle location requests
    tokio::spawn(async move {
        // Allow 4 requests at a time. acquiring a token will block while the
        // number of concurrent location requests is more than this.
        let semaphore = Arc::new(Semaphore::new(4));

        loop {
            while let Ok((id, ip_address)) = rx.recv_async().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let mut response_chan = response_chan.clone();
                let locator = locator.clone();

                // Once we have acquired our permit, spawn a task to avoid
                // blocking this loop so that we can handle concurrent requests.
                tokio::spawn(async move {
                    let location = locator.locate(ip_address).await;
                    let _ = response_chan.send((id, location)).await;

                    // ensure permit is moved into task by dropping it explicitly:
                    drop(permit);
                });
            }
        }
    });

    tx
}

/// This struct can be used to make location requests, given
/// an IPV4 address.
#[derive(Clone)]
struct Locator {
    client: reqwest::Client,
    cache: Arc<RwLock<FxHashMap<Ipv4Addr, Arc<NodeLocation>>>>,
}

impl Locator {
    pub fn new(cache: FxHashMap<Ipv4Addr, Arc<NodeLocation>>) -> Self {
        let client = reqwest::Client::new();

        Locator {
            client,
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub async fn locate(&self, ip: Ipv4Addr) -> Option<Arc<NodeLocation>> {
        // Return location quickly if it's cached:
        let cached_loc = {
            let cache_reader = self.cache.read();
            cache_reader.get(&ip).cloned()
        };
        if cached_loc.is_some() {
            return cached_loc;
        }

        // Look it up via ipapi.co:
        let mut location = self.iplocate_ipapi_co(ip).await;

        // If that fails, try looking it up via ipinfo.co instead:
        if let Err(e) = &location {
            log::warn!(
                "Couldn't obtain location information for {} from ipapi.co: {}",
                ip,
                e
            );
            location = self.iplocate_ipinfo_io(ip).await
        }

        // If both fail, we've logged the errors and we'll return None.
        if let Err(e) = &location {
            log::warn!(
                "Couldn't obtain location information for {} from ipinfo.co: {}",
                ip,
                e
            );
        }

        // If we successfully obtained a location, cache it
        if let Ok(location) = &location {
            self.cache.write().insert(ip, location.clone());
        }

        // Discard the error; we've logged information above.
        location.ok()
    }

    async fn iplocate_ipapi_co(&self, ip: Ipv4Addr) -> Result<Arc<NodeLocation>, anyhow::Error> {
        let location = self.query(&format!("https://ipapi.co/{}/json", ip)).await?;

        Ok(Arc::new(location))
    }

    async fn iplocate_ipinfo_io(&self, ip: Ipv4Addr) -> Result<Arc<NodeLocation>, anyhow::Error> {
        let location = self
            .query::<IPApiLocate>(&format!("https://ipinfo.io/{}/json", ip))
            .await?
            .into_node_location()
            .with_context(|| "Could not convert response into node location")?;

        Ok(Arc::new(location))
    }

    async fn query<T>(&self, url: &str) -> Result<T, anyhow::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        let res = self
            .client
            .get(url)
            .send()
            .await?
            .bytes()
            .await
            .with_context(|| "Failed to obtain response body")?;

        serde_json::from_slice(&res)
            .with_context(|| format!{"Failed to decode '{}'", std::str::from_utf8(&res).unwrap_or("INVALID_UTF8")})
    }
}

/// This is the format returned from ipinfo.co, so we do
/// a little conversion to get it into the shape we want.
#[derive(Deserialize, Debug, Clone)]
struct IPApiLocate {
    city: Box<str>,
    loc: Box<str>,
}

impl IPApiLocate {
    fn into_node_location(self) -> Option<NodeLocation> {
        let IPApiLocate { city, loc } = self;

        let mut loc = loc.split(',').map(|n| n.parse());

        let latitude = loc.next()?.ok()?;
        let longitude = loc.next()?.ok()?;

        // Guarantee that the iterator has been exhausted
        if loc.next().is_some() {
            return None;
        }

        Some(NodeLocation {
            latitude,
            longitude,
            city,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipapi_locate_to_node_location() {
        let ipapi = IPApiLocate {
            loc: "12.5,56.25".into(),
            city: "Foobar".into(),
        };

        let location = ipapi.into_node_location().unwrap();

        assert_eq!(location.latitude, 12.5);
        assert_eq!(location.longitude, 56.25);
        assert_eq!(&*location.city, "Foobar");
    }

    #[test]
    fn ipapi_locate_to_node_location_too_many() {
        let ipapi = IPApiLocate {
            loc: "12.5,56.25,1.0".into(),
            city: "Foobar".into(),
        };

        let location = ipapi.into_node_location();

        assert!(location.is_none());
    }
}
