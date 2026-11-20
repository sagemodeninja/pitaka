use std::slice::Iter;

use crate::core::document::collection::location::RecordLocation;

type Entry = (u64, RecordLocation);

pub struct RecordDirectory {
    directory: Vec<Entry>,
}

impl RecordDirectory {
    pub fn new() -> Self {
        Self { directory: vec![] }
    }

    pub fn insert(&mut self, id: u64, loc: RecordLocation) {
        self.directory.push((id, loc));
    }

    pub fn iter(&self) -> Iter<'_, Entry> {
        self.directory.iter()
    }

    /// Returns the directory entry for the given ID, if it exists.
    pub fn get(&self, id: &u64) -> Option<&Entry> {
        let index = self
            .directory
            .binary_search_by_key(id, |&(entry_id, _)| entry_id)
            .ok()?;
        Some(&self.directory[index])
    }

    /// Returns the location of the record with the given ID, if it exists.
    pub fn find(&self, id: &u64) -> Option<&RecordLocation> {
        let index = self
            .directory
            .binary_search_by_key(id, |&(entry_id, _)| entry_id)
            .ok()?;
        Some(&self.directory[index].1)
    }
}
