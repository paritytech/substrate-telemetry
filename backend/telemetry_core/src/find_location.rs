use std::net::Ipv4Addr;
use std::sync::Arc;

use futures::channel::mpsc;
use futures::{Sink, SinkExt, StreamExt};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use common::node_types::NodeLocation;
use tokio::sync::Semaphore;

/// The returned location is optional; it may be None if not found.
pub type Location = Option<Arc<NodeLocation>>;

/// This is responsible for taking an IP address and attempting
/// to find a geographical location from this
pub fn find_location<Id, R>(response_chan: R) -> mpsc::UnboundedSender<(Id, Ipv4Addr)>
where
    R: Sink<(Id, Option<Arc<NodeLocation>>)> + Unpin + Send + Clone + 'static,
    Id: Clone + Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded();

    // cache entries
    let mut cache: FxHashMap<Ipv4Addr, Option<Arc<NodeLocation>>> = FxHashMap::default();

    // Default entry for localhost
    cache.insert(
        Ipv4Addr::new(127, 0, 0, 1),
        Some(Arc::new(NodeLocation {
            latitude: 52.516_6667,
            longitude: 13.4,
            city: "Berlin".into(),
        })),
    );

    // Create a locator with our cache. This is used to obtain locations.
    let locator = Locator::new(cache);

    // Spawn a loop to handle location requests
    tokio::spawn(async move {
        // Allow 4 requests at a time. acquiring a token will block while the
        // number of concurrent location requests is more than this.
        let semaphore = Arc::new(Semaphore::new(4));

        loop {
            while let Some((id, ip_address)) = rx.next().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let mut response_chan = response_chan.clone();
                let locator = locator.clone();

                // Once we have acquired our permit, spawn a task to avoid
                // blocking this loop so that we can handle concurrent requests.
                tokio::spawn(async move {
                    match locator.locate(ip_address).await {
                        Ok(loc) => {
                            let _ = response_chan.send((id, loc)).await;
                        }
                        Err(e) => {
                            log::debug!("GET error for ip location: {:?}", e);
                        }
                    };

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
    cache: Arc<RwLock<FxHashMap<Ipv4Addr, Option<Arc<NodeLocation>>>>>,
}

impl Locator {
    pub fn new(cache: FxHashMap<Ipv4Addr, Option<Arc<NodeLocation>>>) -> Self {
        let client = reqwest::Client::new();

        Locator {
            client,
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub async fn locate(&self, ip: Ipv4Addr) -> Result<Option<Arc<NodeLocation>>, reqwest::Error> {
        // Return location quickly if it's cached:
        let cached_loc = {
            let cache_reader = self.cache.read();
            cache_reader.get(&ip).map(|o| o.clone())
        };
        if let Some(loc) = cached_loc {
            return Ok(loc);
        }

        // Look it up via the location services if not cached:
        let location = self.iplocate_ipapi_co(ip).await?;
        let location = match location {
            Some(location) => Ok(Some(location)),
            None => self.iplocate_ipinfo_io(ip).await,
        }?;

        self.cache.write().insert(ip, location.clone());
        Ok(location)
    }

    async fn iplocate_ipapi_co(
        &self,
        ip: Ipv4Addr,
    ) -> Result<Option<Arc<NodeLocation>>, reqwest::Error> {
        let location = self
            .query(&format!("https://ipapi.co/{}/json", ip))
            .await?
            .map(Arc::new);

        Ok(location)
    }

    async fn iplocate_ipinfo_io(
        &self,
        ip: Ipv4Addr,
    ) -> Result<Option<Arc<NodeLocation>>, reqwest::Error> {
        let location = self
            .query(&format!("https://ipinfo.io/{}/json", ip))
            .await?
            .and_then(|loc: IPApiLocate| loc.into_node_location().map(Arc::new));

        Ok(location)
    }

    async fn query<T>(&self, url: &str) -> Result<Option<T>, reqwest::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        match self.client.get(url).send().await?.json::<T>().await {
            Ok(result) => Ok(Some(result)),
            Err(err) => {
                log::debug!("JSON error for ip location: {:?}", err);
                Ok(None)
            }
        }
    }
}

/// This is the format returned from ipinfo.co, so we do
/// a little conversion to get it into the shape we want.
#[derive(Deserialize)]
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
