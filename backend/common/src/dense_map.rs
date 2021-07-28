/// This stores items in contiguous memory, making a note of free
/// slots when items are removed again so that they can be reused.
///
/// This is particularly efficient when items are often added and
/// seldom removed.
///
/// Items are keyed by an Id, which can be any type you wish, but
/// must be convertible to/from a `usize`. This promotes using a
/// custom Id type to talk about items in the map.
pub struct DenseMap<Id, T> {
    /// List of retired indexes that can be re-used
    retired: Vec<usize>,
    /// All items
    items: Vec<Option<T>>,
    /// Our ID type
    _id_type: std::marker::PhantomData<Id>,
}

impl<Id, T> DenseMap<Id, T>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    pub fn new() -> Self {
        DenseMap {
            retired: Vec::new(),
            items: Vec::new(),
            _id_type: std::marker::PhantomData,
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
                let id_out = id.into();
                self.items[id] = Some(f(id_out));
                id_out
            }
            None => {
                let id = self.items.len().into();
                self.items.push(Some(f(id)));
                id
            }
        }
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        let id: usize = id.into();
        self.items.get(id).and_then(|item| item.as_ref())
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let id: usize = id.into();
        self.items.get_mut(id).and_then(|item| item.as_mut())
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let id: usize = id.into();
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
            .filter_map(|(id, item)| Some((id.into(), item.as_ref()?)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> + '_ {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(id, item)| Some((id.into(), item.as_mut()?)))
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Id, T)> {
        self.items
            .into_iter()
            .enumerate()
            .filter_map(|(id, item)| Some((id.into(), item?)))
    }

    pub fn len(&self) -> usize {
        self.items.len() - self.retired.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the next Id that will be assigned.
    pub fn next_id(&self) -> usize {
        match self.retired.last() {
            Some(id) => *id,
            None => self.items.len(),
        }
    }
}
