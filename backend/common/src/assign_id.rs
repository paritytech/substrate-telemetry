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

use bimap::BiMap;
use std::hash::Hash;

/// A struct that allows you to assign an Id to an arbitrary set of
/// details (so long as they are Eq+Hash+Clone), and then access
/// the assigned Id given those details or access the details given
/// the Id.
///
/// The Id can be any type that's convertible to/from a `usize`. Using
/// a custom type is recommended for increased type safety.
#[derive(Debug)]
pub struct AssignId<Id, Details> {
    current_id: usize,
    mapping: BiMap<usize, Details>,
    _id_type: std::marker::PhantomData<Id>,
}

impl<Id, Details> Default for AssignId<Id, Details>
where
    Details: Eq + Hash,
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, Details> AssignId<Id, Details>
where
    Details: Eq + Hash,
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    pub fn new() -> Self {
        Self {
            current_id: 0,
            mapping: BiMap::new(),
            _id_type: std::marker::PhantomData,
        }
    }

    pub fn assign_id(&mut self, details: Details) -> Id {
        let this_id = self.current_id;
        self.current_id += 1;
        self.mapping.insert(this_id, details);
        this_id.into()
    }

    pub fn get_details(&mut self, id: Id) -> Option<&Details> {
        self.mapping.get_by_left(&id.into())
    }

    pub fn get_id(&mut self, details: &Details) -> Option<Id> {
        self.mapping.get_by_right(details).map(|&id| id.into())
    }

    pub fn remove_by_id(&mut self, id: Id) -> Option<Details> {
        self.mapping
            .remove_by_left(&id.into())
            .map(|(_, details)| details)
    }

    pub fn remove_by_details(&mut self, details: &Details) -> Option<Id> {
        self.mapping
            .remove_by_right(details)
            .map(|(id, _)| id.into())
    }

    pub fn clear(&mut self) {
        *self = AssignId::new();
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &Details)> {
        self.mapping
            .iter()
            .map(|(&id, details)| (id.into(), details))
    }
}
