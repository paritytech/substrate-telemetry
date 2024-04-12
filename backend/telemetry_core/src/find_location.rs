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

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use futures::{Sink, SinkExt};
use maxminddb::{geoip2::City, Reader as GeoIpReader};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use common::node_types::NodeLocation;

/// The returned location is optional; it may be None if not found.
pub type Location = Option<Arc<NodeLocation>>;

/// This is responsible for taking an IP address and attempting
/// to find a geographical location from this
pub fn find_location<Id, R>(response_chan: R) -> flume::Sender<(Id, IpAddr)>
where
    R: Sink<(Id, Option<Arc<NodeLocation>>)> + Unpin + Send + Clone + 'static,
    Id: Clone + Send + 'static,
{
    let (tx, rx) = flume::unbounded();

    // cache entries
    let mut cache: FxHashMap<IpAddr, Arc<NodeLocation>> = FxHashMap::default();

    // Default entry for localhost
    cache.insert(
        Ipv4Addr::new(127, 0, 0, 1).into(),
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
        loop {
            while let Ok((id, ip_address)) = rx.recv_async().await {
                let mut response_chan = response_chan.clone();
                let locator = locator.clone();

                tokio::spawn(async move {
                    let location = tokio::task::spawn_blocking(move || locator.locate(ip_address))
                        .await
                        .expect("Locate never panics");
                    let _ = response_chan.send((id, location)).await;
                });
            }
        }
    });

    tx
}

/// This struct can be used to make location requests, given
/// an IPV4 or IPV6 address.
#[derive(Debug, Clone)]
struct Locator {
    city: Arc<maxminddb::Reader<&'static [u8]>>,
    cache: Arc<RwLock<FxHashMap<IpAddr, Arc<NodeLocation>>>>,
}

impl Locator {
    /// GeoLite database release data: 2024-03-29
    /// Database and Contents Copyright (c) 2024 MaxMind, Inc.
    /// To download the latest version visit: https://dev.maxmind.com/geoip/geolite2-free-geolocation-data.
    ///
    /// Use of this MaxMind product is governed by MaxMind's GeoLite2 End User License Agreement,
    /// which can be viewed at https://www.maxmind.com/en/geolite2/eula.
    /// This database incorporates GeoNames [https://www.geonames.org] geographical data,
    /// which is made available under the Creative Commons Attribution 4.0 License.
    /// To view a copy of this license, visit https://creativecommons.org/licenses/by/4.0/.
    const CITY_DATA: &'static [u8] = include_bytes!("GeoLite2-City.mmdb");

    pub fn new(cache: FxHashMap<IpAddr, Arc<NodeLocation>>) -> Self {
        Self {
            city: GeoIpReader::from_source(Self::CITY_DATA)
                .map(Arc::new)
                .expect("City data is always valid"),
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub fn locate(&self, ip: IpAddr) -> Option<Arc<NodeLocation>> {
        // Return location quickly if it's cached:
        let cached_loc = {
            let cache_reader = self.cache.read();
            cache_reader.get(&ip).cloned()
        };
        if cached_loc.is_some() {
            return cached_loc;
        }

        let City { city, location, .. } = self.city.lookup(ip.into()).ok()?;
        let city = city
            .as_ref()?
            .names
            .as_ref()?
            .get("en")?
            .to_string()
            .into_boxed_str();
        let latitude = location.as_ref()?.latitude? as f32;
        let longitude = location?.longitude? as f32;

        let location = Arc::new(NodeLocation {
            city,
            latitude,
            longitude,
        });
        self.cache.write().insert(ip, Arc::clone(&location));

        Some(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_construction() {
        Locator::new(Default::default());
    }

    #[test]
    fn locate_random_ip() {
        let ip = "12.5.56.25".parse().unwrap();
        let node_location = Locator::new(Default::default()).locate(ip).unwrap();
        assert_eq!(&*node_location.city, "Gardena");
    }
}
