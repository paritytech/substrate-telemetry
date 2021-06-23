use std::hash::Hash;
use serde::{Serialize,Deserialize};
use bimap::BiMap;

#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq,Serialize,Deserialize)]
pub struct Id(usize);

impl std::convert::From<Id> for usize {
    fn from(id: Id) -> usize {
        id.0
    }
}
impl std::convert::From<usize> for Id {
    fn from(n: usize) -> Id {
        Id(n)
    }
}

/// A struct that allows you to assign ID to an arbitrary set of
/// details (so long as they are Eq+Hash+Clone), and then access
/// the assigned ID given those details or access the details given
/// the ID.
#[derive(Debug)]
pub struct AssignId<Details> {
    current_id: Id,
    mapping: BiMap<Id, Details>
}

impl <Details> AssignId<Details> where Details: Eq + Hash {
    pub fn new() -> Self {
        Self {
            current_id: Id(0),
            mapping: BiMap::new()
        }
    }

    pub fn assign_id(&mut self, details: Details) -> Id {
        let this_id = self.current_id;
        self.current_id.0 += 1;
        self.mapping.insert(this_id, details);
        this_id
    }

    pub fn get_details(&mut self, id: Id) -> Option<&Details> {
        self.mapping.get_by_left(&id)
    }

    pub fn get_id(&mut self, details: &Details) -> Option<Id> {
        self.mapping.get_by_right(details).map(|id| *id)
    }

    pub fn remove_by_id(&mut self, id: Id) -> Option<Details> {
        self.mapping.remove_by_left(&id).map(|(_,details)| details)
    }

    pub fn remove_by_details(&mut self, details: &Details) -> Option<Id> {
        self.mapping.remove_by_right(&details).map(|(id,_)| id)
    }

    pub fn clear(&mut self) {
        *self = AssignId::new();
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &Details)> {
        self.mapping.iter().map(|(id, details)| (*id, details))
    }
}