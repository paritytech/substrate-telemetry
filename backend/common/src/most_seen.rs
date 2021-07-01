use std::collections::HashMap;
use std::hash::Hash;

/// Add items to this, and it will keep track of what the item
/// seen the most is.
#[derive(Debug)]
pub struct MostSeen<T> {
    current_best: T,
    current_count: usize,
    others: HashMap<T, usize>,
}

impl<T: Default> Default for MostSeen<T> {
    fn default() -> Self {
        Self {
            current_best: T::default(),
            current_count: 0,
            others: HashMap::new(),
        }
    }
}

impl<T> MostSeen<T> {
    pub fn new(item: T) -> Self {
        Self {
            current_best: item,
            current_count: 1,
            others: HashMap::new(),
        }
    }
    pub fn best(&self) -> &T {
        &self.current_best
    }
    pub fn best_count(&self) -> usize {
        self.current_count
    }
}

impl<T: Hash + Eq + Clone> MostSeen<T> {
    pub fn insert(&mut self, item: &T) -> ChangeResult {
        if &self.current_best == item {
            // Item already the best one; bump count.
            self.current_count += 1;
            return ChangeResult::NoChange;
        }

        // Item not the best; increment count in map
        let item_count = self.others.entry(item.clone()).or_default();
        *item_count += 1;

        // Is item now the best?
        if *item_count > self.current_count {
            let (mut item, mut count) = self.others.remove_entry(item).expect("item added above");

            // Swap the current best for the new best:
            std::mem::swap(&mut item, &mut self.current_best);
            std::mem::swap(&mut count, &mut self.current_count);

            // Insert the old best back into the map:
            self.others.insert(item, count);

            ChangeResult::NewMostSeenItem
        } else {
            ChangeResult::NoChange
        }
    }
    pub fn remove(&mut self, item: &T) -> ChangeResult {
        if &self.current_best == item {
            // Item already the best one; reduce count (don't allow to drop below 0)
            self.current_count = self.current_count.saturating_sub(1);

            // Is there a new best?
            let other_best = self.others.iter().max_by_key(|f| f.1);

            let (other_item, &other_count) = match other_best {
                Some(item) => item,
                None => return ChangeResult::NoChange,
            };

            if other_count > self.current_count {
                // Clone item to unborrow self.others so that we can remove
                // the item from it. We could pre-emptively remove and reinsert
                // instead, but most of the time there is no change, so I'm
                // aiming to keep that path cheaper.
                let other_item = other_item.clone();
                let (mut other_item, mut other_count) = self
                    .others
                    .remove_entry(&other_item)
                    .expect("item returned above, so def exists");

                // Swap the current best for the new best:
                std::mem::swap(&mut other_item, &mut self.current_best);
                std::mem::swap(&mut other_count, &mut self.current_count);

                // Insert the old best back into the map:
                self.others.insert(other_item, other_count);

                return ChangeResult::NewMostSeenItem;
            } else {
                return ChangeResult::NoChange;
            }
        }

        // Item is in the map; not the best anyway. decrement count.
        if let Some(count) = self.others.get_mut(item) {
            *count += 1;
        }
        ChangeResult::NoChange
    }
}

/// Record the result of adding/removing an entry
#[derive(Clone, Copy)]
pub enum ChangeResult {
    /// The best item has remained the same.
    NoChange,
    /// There is a new best item now.
    NewMostSeenItem,
}

impl ChangeResult {
    pub fn has_changed(self) -> bool {
        match self {
            ChangeResult::NewMostSeenItem => true,
            ChangeResult::NoChange => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_renames_instantly() {
        let mut a: MostSeen<&str> = MostSeen::default();
        let res = a.insert(&"Hello");
        assert_eq!(*a.best(), "Hello");
        assert!(res.has_changed());
    }

    #[test]
    fn new_renames_on_second_change() {
        let mut a: MostSeen<&str> = MostSeen::new("First");
        a.insert(&"Second");
        assert_eq!(*a.best(), "First");
        a.insert(&"Second");
        assert_eq!(*a.best(), "Second");
    }

    #[test]
    fn removing_doesnt_underflow() {
        let mut a: MostSeen<&str> = MostSeen::new("First");
        a.remove(&"First");
        a.remove(&"First");
        a.remove(&"Second");
        a.remove(&"Third");
    }

    #[test]
    fn keeps_track_of_best_count() {
        let mut a: MostSeen<&str> = MostSeen::default();
        a.insert(&"First");
        assert_eq!(a.best_count(), 1);

        a.insert(&"First");
        assert_eq!(a.best_count(), 2);

        a.insert(&"First");
        assert_eq!(a.best_count(), 3);

        a.remove(&"First");
        assert_eq!(a.best_count(), 2);

        a.remove(&"First");
        assert_eq!(a.best_count(), 1);

        a.remove(&"First");
        assert_eq!(a.best_count(), 0);

        a.remove(&"First");
        assert_eq!(a.best_count(), 0);
    }

    #[test]
    fn it_tracks_best_on_insert() {
        let mut a: MostSeen<&str> = MostSeen::default();

        a.insert(&"First");
        assert_eq!(*a.best(), "First", "1");

        a.insert(&"Second");
        assert_eq!(*a.best(), "First", "2");

        a.insert(&"Second");
        assert_eq!(*a.best(), "Second", "3");

        a.insert(&"First");
        assert_eq!(*a.best(), "Second", "4");

        a.insert(&"First");
        assert_eq!(*a.best(), "First", "5");
    }

    #[test]
    fn it_tracks_best() {
        let mut a: MostSeen<&str> = MostSeen::default();
        a.insert(&"First");
        a.insert(&"Second");
        a.insert(&"Third"); // 1

        a.insert(&"Second");
        a.insert(&"Second"); // 3
        a.insert(&"First"); // 2

        assert_eq!(*a.best(), "Second");
        assert_eq!(a.best_count(), 3);

        let res = a.remove(&"Second");

        assert!(!res.has_changed());
        assert_eq!(a.best_count(), 2);
        assert_eq!(*a.best(), "Second"); // Tied with "First"

        let res = a.remove(&"Second");

        assert!(res.has_changed());
        assert_eq!(a.best_count(), 2);
        assert_eq!(*a.best(), "First"); // First is now ahead
    }
}
