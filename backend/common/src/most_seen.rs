use std::collections::HashMap;
use std::hash::Hash;

/// Add items to this, and it will keep track of what the item
/// seen the most is.
#[derive(Debug)]
pub struct MostSeen<T> {
    current_best: T,
    current_count: usize,
    others: HashMap<T, usize>
}

impl <T: Hash + Eq> MostSeen<T> {
    pub fn new(item: T) -> Self {
        Self {
            current_best: item,
            current_count: 1,
            others: HashMap::new()
        }
    }
    pub fn best(&self) -> &T {
        &self.current_best
    }
}

impl <T: Hash + Eq + Clone> MostSeen<T> {
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
            let (item, count) = self.others
                .remove_entry(item)
                .expect("item added above");
            self.current_best = item;
            self.current_count = count;

            ChangeResult::NewMostSeenItem
        } else {
            ChangeResult::NoChange
        }
    }
    pub fn remove(&mut self, item: &T) -> ChangeResult {
        if &self.current_best == item {
            // Item already the best one; reduce count
            self.current_count -= 1;

            // Is there a new best?
            let other_best = self.others
                .iter()
                .max_by_key(|f| f.1);

            let (other_item, &other_count) = match other_best {
                Some(item) => item,
                None => { return ChangeResult::NoChange }
            };

            if other_count > self.current_count {
                // Clone item to unborrow self.others so that we can remove
                // the item from it. We could pre-emptively remove and reinsert
                // instead, but most of the time there is no change, so I'm
                // aiming to keep that path cheaper.
                let other_item = other_item.clone();
                let (other_item, other_count) = self.others
                    .remove_entry(&other_item)
                    .expect("item returned above, so def exists");

                self.current_best = other_item;
                self.current_count = other_count;

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
#[derive(Clone,Copy)]
pub enum ChangeResult {
    /// The best item has remained the same.
    NoChange,
    /// There is a new best item now.
    NewMostSeenItem
}

impl ChangeResult {
    pub fn has_changed(self) -> bool {
        match self {
            ChangeResult::NewMostSeenItem => true,
            ChangeResult::NoChange => false
        }
    }
}