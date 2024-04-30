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

use num_traits::{SaturatingAdd, SaturatingSub, Zero};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Build an object responsible for keeping track of a rolling total.
/// It does this in constant time and using memory proportional to the
/// granularity * window size multiple that we set.
pub struct RollingTotalBuilder<Time: TimeSource = SystemTimeSource> {
    window_size_multiple: usize,
    granularity: Duration,
    time_source: Time,
}

impl RollingTotalBuilder {
    /// Build a [`RollingTotal`] struct. By default,
    /// the window_size is 10s, the granularity is 1s,
    /// and system time is used.
    pub fn new() -> RollingTotalBuilder<SystemTimeSource> {
        Self {
            window_size_multiple: 10,
            granularity: Duration::from_secs(1),
            time_source: SystemTimeSource,
        }
    }

    /// Set the source of time we'll use. By default, we use system time.
    pub fn time_source<Time: TimeSource>(self, val: Time) -> RollingTotalBuilder<Time> {
        RollingTotalBuilder {
            window_size_multiple: self.window_size_multiple,
            granularity: self.granularity,
            time_source: val,
        }
    }

    /// Set the size of the window of time that we'll look back on
    /// to sum up values over to give us the current total. The size
    /// is set as a multiple of the granularity; a granularity of 1s
    /// and a size of 10 means the window size will be 10 seconds.
    pub fn window_size_multiple(mut self, val: usize) -> Self {
        self.window_size_multiple = val;
        self
    }

    /// What is the granularity of our windows of time. For example, a
    /// granularity of 5 seconds means that every 5 seconds the window
    /// that we look at shifts forward to the next 5 seconds worth of data.
    /// A larger granularity is more efficient but less accurate than a
    /// smaller one.
    pub fn granularity(mut self, val: Duration) -> Self {
        self.granularity = val;
        self
    }
}

impl<Time: TimeSource> RollingTotalBuilder<Time> {
    /// Create a [`RollingTotal`] with these settings, starting from the
    /// instant provided.
    pub fn start<T>(self) -> RollingTotal<T, Time>
    where
        T: Zero + SaturatingAdd + SaturatingSub,
    {
        let mut averages = VecDeque::new();
        averages.push_back((self.time_source.now(), T::zero()));

        RollingTotal {
            window_size_multiple: self.window_size_multiple,
            time_source: self.time_source,
            granularity: self.granularity,
            averages,
            total: T::zero(),
        }
    }
}

pub struct RollingTotal<Val, Time = SystemTimeSource> {
    window_size_multiple: usize,
    time_source: Time,
    granularity: Duration,
    averages: VecDeque<(Instant, Val)>,
    total: Val,
}

impl<Val, Time: TimeSource> RollingTotal<Val, Time>
where
    Val: SaturatingAdd + SaturatingSub + Copy + std::fmt::Debug,
    Time: TimeSource,
{
    /// Add a new value at some time.
    pub fn push(&mut self, value: Val) {
        let time = self.time_source.now();
        let (last_time, last_val) = self.averages.back_mut().expect("always 1 value");

        let since_last_nanos = time.duration_since(*last_time).as_nanos();
        let granularity_nanos = self.granularity.as_nanos();

        if since_last_nanos >= granularity_nanos {
            // New time doesn't fit into last bucket; create a new bucket with a time
            // that is some number of granularity steps from the last, and add the
            // value to that.

            // This rounds down, eg 7 / 5 = 1. Find the number of granularity steps
            // to jump from the last time such that the jump can fit this new value.
            let steps = since_last_nanos / granularity_nanos;

            // Create a new time this number of jumps forward, and push it.
            let new_time =
                *last_time + Duration::from_nanos(granularity_nanos as u64) * steps as u32;
            self.total = self.total.saturating_add(&value);
            self.averages.push_back((new_time, value));

            // Remove any old times/values no longer within our window size. If window_size_multiple
            // is 1, then we only keep the just-pushed time, hence the "-1". Remember to keep our
            // cached total up to date if we remove things.
            let oldest_time_in_window =
                new_time - (self.granularity * (self.window_size_multiple - 1) as u32);
            while self.averages.front().expect("always 1 value").0 < oldest_time_in_window {
                let value = self.averages.pop_front().expect("always 1 value").1;
                self.total = self.total.saturating_sub(&value);
            }
        } else {
            // New time fits into our last bucket, so just add it on. We don't need to worry
            // about bucket cleanup since number/times of buckets hasn't changed.
            *last_val = last_val.saturating_add(&value);
            self.total = self.total.saturating_add(&value);
        }
    }

    /// Fetch the current rolling total that we've accumulated. Note that this
    /// is based on the last seen times and values, and is not affected by the time
    /// that it is called.
    pub fn total(&self) -> Val {
        self.total
    }

    /// Fetch the current time source, in case we need to modify it.
    pub fn time_source(&mut self) -> &mut Time {
        &mut self.time_source
    }

    #[cfg(test)]
    pub fn averages(&self) -> &VecDeque<(Instant, Val)> {
        &self.averages
    }
}

/// A source of time that we can use in our rolling total.
/// This allows us to avoid explicitly mentioning time when pushing
/// new values, and makes it a little harder to accidentally pass
/// an older time and cause a panic.
pub trait TimeSource {
    fn now(&self) -> Instant;
}

pub struct SystemTimeSource;
impl TimeSource for SystemTimeSource {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

pub struct UserTimeSource(Instant);
impl UserTimeSource {
    pub fn new(time: Instant) -> Self {
        UserTimeSource(time)
    }
    pub fn set_time(&mut self, time: Instant) {
        self.0 = time;
    }
    pub fn increment_by(&mut self, duration: Duration) {
        self.0 += duration;
    }
}
impl TimeSource for UserTimeSource {
    fn now(&self) -> Instant {
        self.0
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn deosnt_grow_beyond_window_size() {
        let start_time = Instant::now();
        let granularity = Duration::from_secs(1);
        let mut rolling_total = RollingTotalBuilder::new()
            .granularity(granularity)
            .window_size_multiple(3) // There should be no more than 3 buckets ever,
            .time_source(UserTimeSource(start_time))
            .start();

        for n in 0..1_000 {
            rolling_total.push(n);
            rolling_total
                .time_source()
                .increment_by(Duration::from_millis(300)); // multiple values per granularity.
        }

        assert_eq!(rolling_total.averages().len(), 3);
        assert!(rolling_total.averages().capacity() < 10); // Just to show that it's capacity is bounded.
    }

    #[test]
    fn times_grouped_by_granularity_spacing() {
        let start_time = Instant::now();
        let granularity = Duration::from_secs(1);
        let mut rolling_total = RollingTotalBuilder::new()
            .granularity(granularity)
            .window_size_multiple(10)
            .time_source(UserTimeSource(start_time))
            .start();

        rolling_total.push(1);

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(1210)); // 1210; bucket 1
        rolling_total.push(2);

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(2500)); // 3710: bucket 3
        rolling_total.push(3);

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(1100)); // 4810: bucket 4
        rolling_total.push(4);

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(190)); // 5000: bucket 5
        rolling_total.push(5);

        // Regardless of the exact time that's elapsed, we'll end up with buckets that
        // are exactly granularity spacing (or multiples of) apart.
        assert_eq!(
            rolling_total
                .averages()
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![
                (start_time, 1),
                (start_time + granularity, 2),
                (start_time + granularity * 3, 3),
                (start_time + granularity * 4, 4),
                (start_time + granularity * 5, 5),
            ]
        )
    }

    #[test]
    fn gets_correct_total_within_granularity() {
        let start_time = Instant::now();
        let mut rolling_total = RollingTotalBuilder::new()
            .granularity(Duration::from_secs(1))
            .window_size_multiple(10)
            .time_source(UserTimeSource(start_time))
            .start();

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(300));
        rolling_total.push(1);

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(300));
        rolling_total.push(10);

        rolling_total
            .time_source()
            .increment_by(Duration::from_millis(300));
        rolling_total.push(-5);

        assert_eq!(rolling_total.total(), 6);
        assert_eq!(rolling_total.averages().len(), 1);
    }

    #[test]
    fn gets_correct_total_within_window() {
        let start_time = Instant::now();
        let mut rolling_total = RollingTotalBuilder::new()
            .granularity(Duration::from_secs(1))
            .window_size_multiple(10)
            .time_source(UserTimeSource(start_time))
            .start();

        rolling_total.push(4);

        assert_eq!(rolling_total.averages().len(), 1);
        assert_eq!(rolling_total.total(), 4);

        rolling_total
            .time_source()
            .increment_by(Duration::from_secs(3));
        rolling_total.push(1);

        assert_eq!(rolling_total.averages().len(), 2);
        assert_eq!(rolling_total.total(), 5);

        rolling_total
            .time_source()
            .increment_by(Duration::from_secs(1));
        rolling_total.push(10);

        assert_eq!(rolling_total.averages().len(), 3);
        assert_eq!(rolling_total.total(), 15);

        // Jump precisely to the end of the window. Now, pushing a
        // value will displace the first one (4). Note: if no value
        // is pushed, this time change will have no effect.
        rolling_total
            .time_source()
            .increment_by(Duration::from_secs(8));
        rolling_total.push(20);

        assert_eq!(rolling_total.averages().len(), 3);
        assert_eq!(rolling_total.total(), 15 + 20 - 4);

        // Jump so that only the last value is still within the window:
        rolling_total
            .time_source()
            .increment_by(Duration::from_secs(9));
        rolling_total.push(1);

        assert_eq!(rolling_total.averages().len(), 2);
        assert_eq!(rolling_total.total(), 21);

        // Jump so that everything is out of scope (just about!):
        rolling_total
            .time_source()
            .increment_by(Duration::from_secs(10));
        rolling_total.push(1);

        assert_eq!(rolling_total.averages().len(), 1);
        assert_eq!(rolling_total.total(), 1);
    }
}
