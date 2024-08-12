// Source code for the Substrate Telemetry Server.
// Copyright (C) 2022 Parity Technologies (UK) Ltd.
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

use crate::feed_message::Ranking;
use std::collections::HashMap;

/// A data structure which counts how many occurrences of a given key we've seen.
#[derive(Default)]
pub struct Counter<K> {
    /// A map containing the number of occurrences of a given key.
    ///
    /// If there are none then the entry is removed.
    map: HashMap<K, u64>,

    /// The number of occurrences where the key is `None`.
    empty: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CounterValue {
    Increment,
    Decrement,
}

impl<K> Counter<K>
where
    K: Sized + std::hash::Hash + Eq,
{
    /// Either adds or removes a single occurence of a given `key`.
    pub fn modify<'a, Q>(&mut self, key: Option<&'a Q>, op: CounterValue)
    where
        Q: ?Sized + std::hash::Hash + Eq,
        K: std::borrow::Borrow<Q>,
        Q: std::borrow::ToOwned<Owned = K>,
    {
        if let Some(key) = key {
            if let Some(entry) = self.map.get_mut(key) {
                match op {
                    CounterValue::Increment => {
                        *entry += 1;
                    }
                    CounterValue::Decrement => {
                        *entry -= 1;
                        if *entry == 0 {
                            // Don't keep entries for which there are no hits.
                            self.map.remove(key);
                        }
                    }
                }
            } else {
                assert_eq!(op, CounterValue::Increment);
                self.map.insert(key.to_owned(), 1);
            }
        } else {
            match op {
                CounterValue::Increment => {
                    self.empty += 1;
                }
                CounterValue::Decrement => {
                    self.empty -= 1;
                }
            }
        }
    }

    /// Generates a top-N table of the most common keys.
    pub fn generate_ranking_top(&self, max_count: usize) -> Ranking<K>
    where
        K: Clone,
    {
        let mut all: Vec<(&K, u64)> = self.map.iter().map(|(key, count)| (key, *count)).collect();
        all.sort_unstable_by_key(|&(_, count)| !count);

        let list = all
            .iter()
            .take(max_count)
            .map(|&(key, count)| (key.clone(), count))
            .collect();

        let other = all
            .iter()
            .skip(max_count)
            .fold(0, |sum, (_, count)| sum + *count);

        Ranking {
            list,
            other,
            unknown: self.empty,
        }
    }

    /// Generates a sorted table of all of the keys.
    pub fn generate_ranking_ordered(&self) -> Ranking<K>
    where
        K: Copy + Clone + Ord,
    {
        let mut list: Vec<(K, u64)> = self.map.iter().map(|(key, count)| (*key, *count)).collect();
        list.sort_unstable_by_key(|&(key, count)| (key, !count));

        Ranking {
            list,
            other: 0,
            unknown: self.empty,
        }
    }
}
