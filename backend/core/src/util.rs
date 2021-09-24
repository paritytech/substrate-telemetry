mod dense_map;
mod hash;
mod location;
mod mean_list;
mod num_stats;

pub use dense_map::DenseMap;
pub use hash::{Hash, HashParseError};
pub use location::{LocateRequest, Locator, LocatorFactory};
pub use mean_list::MeanList;
pub use num_stats::NumStats;

/// Returns current unix time in ms (compatible with JS Date.now())
pub fn now() -> u64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time must be configured to be post Unix Epoch start; qed")
        .as_millis() as u64
}
