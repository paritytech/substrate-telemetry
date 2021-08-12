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

use num_traits::{Bounded, NumOps, Zero};
use std::convert::TryFrom;
use std::iter::Sum;

/// Keep track of last N numbers pushed onto internal stack.
/// Provides means to get an average of said numbers.
pub struct NumStats<T> {
    stack: Box<[T]>,
    index: usize,
    sum: T,
}

impl<T: NumOps + Zero + Bounded + Copy + Sum + TryFrom<usize>> NumStats<T> {
    pub fn new(size: usize) -> Self {
        NumStats {
            stack: vec![T::zero(); size].into_boxed_slice(),
            index: 0,
            sum: T::zero(),
        }
    }

    pub fn push(&mut self, val: T) {
        let slot = &mut self.stack[self.index % self.stack.len()];

        self.sum = (self.sum + val) - *slot;

        *slot = val;

        self.index += 1;
    }

    pub fn average(&self) -> T {
        let cap = std::cmp::min(self.index, self.stack.len());

        if cap == 0 {
            return T::zero();
        }

        let cap = T::try_from(cap).unwrap_or_else(|_| T::max_value());

        self.sum / cap
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.sum = T::zero();

        for val in self.stack.iter_mut() {
            *val = T::zero();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_correct_average() {
        let mut stats: NumStats<u64> = NumStats::new(10);

        stats.push(3);
        stats.push(7);

        assert_eq!(stats.average(), 5);
    }

    #[test]
    fn calculates_correct_average_over_bounds() {
        let mut stats: NumStats<u64> = NumStats::new(10);

        stats.push(100);

        for _ in 0..9 {
            stats.push(0);
        }

        assert_eq!(stats.average(), 10);

        stats.push(0);

        assert_eq!(stats.average(), 0);
    }

    #[test]
    fn resets_properly() {
        let mut stats: NumStats<u64> = NumStats::new(10);

        for _ in 0..10 {
            stats.push(100);
        }

        assert_eq!(stats.average(), 100);

        stats.reset();

        assert_eq!(stats.average(), 0);

        stats.push(7);
        stats.push(3);

        assert_eq!(stats.average(), 5);
    }
}
