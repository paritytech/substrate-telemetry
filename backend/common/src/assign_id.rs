use std::{collections::HashMap, hash::Hash};
use serde::{Serialize,Deserialize};

#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq,Serialize,Deserialize)]
pub struct Id(usize);

impl std::convert::From<Id> for usize {
    fn from(id: Id) -> usize {
        id.0
    }
}

/// A struct that allows you to assign ID to an arbitrary set of
/// details (so long as they are Eq+Hash+Clone), and then access
/// the assigned ID given those details or access the details given
/// the ID.
#[derive(Debug)]
pub struct AssignId<Details> {
    current_id: Id,
    from_details: HashMap<Details, Id>,
    from_id: HashMap<Id, Details>
}

impl <Details> AssignId<Details> where Details: Eq + Hash + Clone {
    pub fn new() -> Self {
        Self {
            current_id: Id(0),
            from_details: HashMap::new(),
            from_id: HashMap::new()
        }
    }

    pub fn assign_id(&mut self, details: Details) -> Id {
        let this_id = self.current_id;
        self.current_id.0 += 1;

        self.from_details.insert(details.clone(), this_id);
        self.from_id.insert(this_id, details);

        this_id
    }

    pub fn get_details(&mut self, id: Id) -> Option<&Details> {
        self.from_id.get(&id)
    }

    pub fn get_id(&mut self, details: &Details) -> Option<Id> {
        self.from_details.get(details).map(|id| *id)
    }

    pub fn remove_by_id(&mut self, id: Id) -> Option<Details> {
        if let Some(details) = self.from_id.remove(&id) {
            self.from_details.remove(&details);
            Some(details)
        } else {
            None
        }
    }

    pub fn remove_by_details(&mut self, details: &Details) -> Option<Id> {
        if let Some(id) = self.from_details.remove(&details) {
            self.from_id.remove(&id);
            Some(id)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        *self = AssignId::new()
    }
}