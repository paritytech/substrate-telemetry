mod mean_list;
mod location;

pub use mean_list::MeanList;
pub use location::{Locator, LocateRequest, Location};

type Id = usize;

pub struct DenseMap<T> {
    /// List of retired indexes that can be re-used
    retired: Vec<Id>,
    /// All items
    items: Vec<Option<T>>,
}

impl<T> DenseMap<T> {
    pub fn new() -> Self {
        DenseMap {
            retired: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, item: T) -> Id {
        self.add_with(|_| item)
    }

    pub fn add_with<F>(&mut self, f: F) -> Id
    where
        F: FnOnce(Id) -> T,
    {
        match self.retired.pop() {
            Some(id) => {
                self.items[id] = Some(f(id));
                id
            },
            None => {
                let id = self.items.len();
                self.items.push(Some(f(id)));
                id
            },
        }
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        self.items.get(id).and_then(|item| item.as_ref())
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        self.items.get_mut(id).and_then(|item| item.as_mut())
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let old = self.items.get_mut(id).and_then(|item| item.take());

        if old.is_some() {
            // something was actually removed, so lets add the id to
            // the list of retired ids!
            self.retired.push(id);
        }

        old
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> + '_ {
        self.items.iter().enumerate().filter_map(|(id, item)| {
            Some((id, item.as_ref()?))
        })
    }

    pub fn len(&self) -> usize {
        self.items.len() - self.retired.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn fnv<D: AsRef<[u8]>>(data: D) -> u64 {
    use fnv::FnvHasher;
    use std::hash::Hasher;

    let mut hasher = FnvHasher::default();

    hasher.write(data.as_ref());
    hasher.finish()
}

/// Returns current unix time in ms (compatible with JS Date.now())
pub fn now() -> u64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time must be configured to be post Unix Epoch start; qed")
        .as_millis() as u64
}
