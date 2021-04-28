pub type Id = usize;

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
            }
            None => {
                let id = self.items.len();
                self.items.push(Some(f(id)));
                id
            }
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
        self.items
            .iter()
            .enumerate()
            .filter_map(|(id, item)| Some((id, item.as_ref()?)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> + '_ {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(id, item)| Some((id, item.as_mut()?)))
    }

    pub fn len(&self) -> usize {
        self.items.len() - self.retired.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
