mod error;

use std::slice::Iter;

use crate::core::document::collection::{
    directory::error::RecordDirectoryError, location::RecordLocation,
};

type Entry = (u64, RecordLocation);

pub struct RecordDirectory {
    directory: Vec<Entry>,
}

impl RecordDirectory {
    pub fn new() -> Self {
        Self { directory: vec![] }
    }

    pub fn iter(&self) -> Iter<'_, Entry> {
        self.directory.iter()
    }

    pub fn index_of(&self, id: &u64) -> Option<usize> {
        self.directory
            .binary_search_by_key(id, |&(entry_id, _)| entry_id)
            .ok()
    }

    /// Returns the directory entry for the given ID, if it exists.
    pub fn get(&self, id: &u64) -> Option<&Entry> {
        let index = self.index_of(id)?;
        Some(&self.directory[index])
    }

    /// Returns the location of the record with the given ID, if it exists.
    pub fn find(&self, id: &u64) -> Option<&RecordLocation> {
        let index = self.index_of(id)?;
        Some(&self.directory[index].1)
    }

    pub fn insert(&mut self, id: u64, loc: RecordLocation) {
        self.directory.push((id, loc));
    }

    pub fn update(&mut self, id: u64, loc: RecordLocation) -> Result<usize, RecordDirectoryError> {
        match self.index_of(&id) {
            Some(index) => {
                self.directory[index] = (id, loc);
                Ok(index)
            }
            None => Err(RecordDirectoryError::Update(id)),
        }
    }
}
