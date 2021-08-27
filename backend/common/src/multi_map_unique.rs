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
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A map where each key can contain multiple values. We enforce that a value
/// only ever belongs to one key at a time (the latest key it was inserted
/// against).
pub struct MultiMapUnique<K, V> {
    value_to_key: HashMap<V, K>,
    key_to_values: HashMap<K, HashSet<V>>,
}

impl<K, V> MultiMapUnique<K, V> {
    /// Construct a new MultiMap
    pub fn new() -> Self {
        Self {
            value_to_key: HashMap::new(),
            key_to_values: HashMap::new(),
        }
    }

    /// Return the set of values associated with a key.
    pub fn get_values(&self, key: &K) -> Option<&HashSet<V>>
    where
        K: Eq + Hash,
    {
        self.key_to_values.get(key)
    }

    /// Remove a value from the MultiMap, returning the key it was found
    /// under, if it was found at all.
    ///
    /// ```
    /// let mut m = common::MultiMapUnique::new();
    ///
    /// m.insert("a", 1);
    /// m.insert("a", 2);
    ///
    /// m.insert("b", 3);
    /// m.insert("b", 4);
    ///
    /// assert_eq!(m.num_keys(), 2);
    /// assert_eq!(m.num_values(), 4);
    ///
    /// m.remove_value(&1);
    ///
    /// assert_eq!(m.num_keys(), 2);
    /// assert_eq!(m.num_values(), 3);
    ///
    /// m.remove_value(&2);
    ///
    /// assert_eq!(m.num_keys(), 1);
    /// assert_eq!(m.num_values(), 2);
    /// ```
    pub fn remove_value(&mut self, value: &V) -> Option<K>
    where
        V: Eq + Hash,
        K: Eq + Hash,
    {
        if let Some(key) = self.value_to_key.remove(value) {
            if let Some(m) = self.key_to_values.get_mut(&key) {
                m.remove(value);
                if m.is_empty() {
                    self.key_to_values.remove(&key);
                }
            }
            return Some(key);
        }
        None
    }

    /// Insert a key+value pair into the multimap. Multiple different
    /// values can exist for a single key, but only one of each value can
    /// exist in the MultiMap.
    ///
    /// ```
    /// let mut m = common::MultiMapUnique::new();
    ///
    /// m.insert("a", 1);
    /// m.insert("b", 1);
    /// m.insert("c", 1);
    ///
    /// assert_eq!(m.num_keys(), 1);
    /// assert_eq!(m.num_values(), 1);
    ///
    /// // The value `1` must be unique in the map, so it only exists
    /// // in the last location it was inserted:
    /// assert!(m.get_values(&"a").is_none());
    /// assert!(m.get_values(&"b").is_none());
    /// assert_eq!(m.get_values(&"c").unwrap().iter().collect::<Vec<_>>(), vec![&1]);
    /// ```
    pub fn insert(&mut self, key: K, value: V)
    where
        V: Clone + Eq + Hash,
        K: Clone + Eq + Hash,
    {
        // Ensure that the value doesn't exist elsewhere already;
        // values must be unique and only belong to one key:
        self.remove_value(&value);

        self.value_to_key.insert(value.clone(), key.clone());
        self.key_to_values.entry(key).or_default().insert(value);
    }

    /// Number of values stored in the map
    pub fn num_values(&self) -> usize {
        self.value_to_key.len()
    }

    /// Number of keys stored in the map
    pub fn num_keys(&self) -> usize {
        self.key_to_values.len()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn multiple_values_allowed_per_key() {
        let mut m = MultiMapUnique::new();

        m.insert("a", 1);
        m.insert("a", 2);

        m.insert("b", 3);
        m.insert("b", 4);

        assert_eq!(m.num_keys(), 2);
        assert_eq!(m.num_values(), 4);

        let a_vals = m.get_values(&"a").expect("a vals");
        assert!(a_vals.contains(&1));
        assert!(a_vals.contains(&2));

        let b_vals = m.get_values(&"b").expect("b vals");
        assert!(b_vals.contains(&3));
        assert!(b_vals.contains(&4));
    }
}
