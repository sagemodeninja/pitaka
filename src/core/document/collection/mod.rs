mod account;
mod directory;
mod index;
mod query;
mod record;
mod transaction;

pub use account::*;
pub use record::*;
pub use transaction::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Seek};
use std::os::unix::fs::MetadataExt;

use crate::core::document::collection::delta::RecordDelta;
use crate::core::document::collection::directory::RecordDirectory;
use crate::core::document::collection::index::{CollectionIndexBuilder, IndexValue};
use crate::core::document::collection::location::RecordLocation::{self, InMemory, OnDisk};
use crate::core::document::collection::query::CollectionQueryBuilder;
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};
use crate::utils::file::FileHandle;

const COLLECTION_SIZE: u64 = 48;
const FIRST_COLLECTION_OFFSET: u64 = 16;
const INIT_COLLECTION_COUNT: u64 = 2;

pub struct Collection<T: RecordPayload> {
    pub position: u64,

    pub o_next_collection: u64,
    pub record_type: u8,
    pub next_id: u64,
    pub record_count: u64,
    pub o_first_record: u64,
    pub o_last_record: u64,

    dir: RecordDirectory,
    directory: HashMap<u64, RecordLocation>,
    indexes: CollectionIndexBuilder<T>,
    updates: Vec<RecordDelta>,
    tracked: HashMap<u64, Record<T>>,
}

impl<T: RecordPayload> Collection<T> {
    pub fn empty() -> Self {
        Self {
            position: 0,

            o_next_collection: 0,
            record_type: T::record_type(),
            next_id: 1,
            record_count: 0,
            o_first_record: 0,
            o_last_record: 0,

            dir: RecordDirectory::new(),
            directory: HashMap::new(),
            indexes: CollectionIndexBuilder::new(),
            updates: Vec::new(),
            tracked: HashMap::new(),
        }
    }

    pub fn create(file: &mut File) -> io::Result<Self> {
        let mut collection = Self::empty();
        let r_type = collection.record_type as u64;

        collection.position = FIRST_COLLECTION_OFFSET + (r_type - 1) * COLLECTION_SIZE;
        collection.o_next_collection = if r_type < INIT_COLLECTION_COUNT {
            FIRST_COLLECTION_OFFSET + r_type * COLLECTION_SIZE
        } else {
            0
        };

        collection.write_to_file(file)?;
        Ok(collection)
    }

    // See: /docs/develop/file_layout.md
    pub fn read(&mut self, file: &mut File) -> io::Result<()> {
        self.position = file.stream_position()?;

        self.o_next_collection = file.read_le_u64()?;
        self.record_type = file.read_le_u8()?;
        self.next_id = file.read_le_u64()?;
        self.record_count = file.read_le_u64()?;
        self.o_first_record = file.read_le_u64()?;
        self.o_last_record = file.read_le_u64()?;
        // +7 reserved bytes.

        Ok(())
    }

    pub fn add_index<'a>(
        &mut self,
        key: &str,
        eval: impl Fn(&T) -> IndexValue + 'static,
    ) -> &mut CollectionIndexBuilder<T> {
        self.indexes.add_index(key, eval)
    }

    pub fn build_indexes(&mut self, file: &mut File) -> io::Result<()> {
        let mut o_r_next = self.o_first_record;

        for _ in 0..self.record_count {
            file.move_to(o_r_next)?;

            let record = Record::<T>::read_from_file(file)?;

            self.dir.insert(record.id, OnDisk(o_r_next));
            self.directory.insert(record.id, OnDisk(o_r_next));
            self.indexes.insert(&record);

            o_r_next = record.o_next;
        }

        Ok(())
    }

    pub fn query<'a>(&'a self, file: &'a mut File) -> CollectionQueryBuilder<'a, T>
    where
        T: Clone,
    {
        CollectionQueryBuilder::new(move |plan| {
            let mut records = Vec::new();

            let mut candidates: Box<dyn DoubleEndedIterator<Item = &(u64, RecordLocation)> + '_> =
                if plan.indexed_filters.is_empty() {
                    Box::new(self.dir.iter())
                } else {
                    let ids = plan
                        .indexed_filters
                        .iter()
                        .next()
                        .and_then(|(key, value)| self.indexes.get(key)?.get(value))
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);

                    Box::new(ids.iter().filter_map(move |id| self.dir.get(id)))
                };

            let mut reversed;
            let iter: &mut dyn Iterator<Item = &(u64, RecordLocation)> = if plan.is_reversed {
                reversed = candidates.rev();
                &mut reversed
            } else {
                &mut candidates
            };

            let take = match plan.take {
                Some(take) => take as usize,
                None => usize::MAX,
            };

            for (id, location) in iter.skip(plan.skip as usize).take(take) {
                match location {
                    OnDisk(offset) => {
                        file.move_to(*offset)?;
                        records.push(Record::<T>::read_from_file(file)?);
                    }
                    InMemory => {
                        if let Some(item) = self.tracked.get(id) {
                            records.push(item.clone());
                        }
                    }
                }
            }

            Ok(records)
        })
    }

    pub fn add(&mut self, item: T) {
        let id = self.next_id;

        let record = Record::new(self.o_last_record, id, item);

        self.directory.insert(id, InMemory);
        self.indexes.insert(&record);
        self.updates.push(RecordDelta::Create(id));
        self.tracked.insert(id, record);

        self.next_id += 1;
    }
}

impl<T: RecordPayload> FileHandle for Collection<T> {
    fn read_from_file(_: &mut File) -> io::Result<Self> {
        todo!()
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let for_append: Vec<u64> = self
            .updates
            .iter()
            .filter_map(|delta| match delta {
                RecordDelta::Create(id) => Some(*id),
                _ => None,
            })
            .collect();

        let append_len = for_append.len() as u64;
        let record_cnt = self.record_count + append_len;

        if append_len > 0 {
            let mut o_prev = self.o_last_record;
            let mut o_next = file.metadata()?.size();

            if self.record_count == 0 {
                self.o_first_record = o_next;
            } else {
                file.move_to(self.o_last_record + 8)?;
                file.write_le_u64(o_next)?;
            }

            // Append new records...
            for index in 0..append_len {
                let is_last = index == append_len - 1;

                if is_last {
                    self.o_last_record = o_next;
                }

                let id = for_append[index as usize];
                if let Some(record) = self.tracked.get_mut(&id) {
                    file.move_to(o_next)?;

                    record.o_prev = o_prev;

                    o_prev = o_next;
                    o_next = if !is_last {
                        o_next + record.size() as u64
                    } else {
                        0
                    };

                    record.o_next = o_next;
                    record.write_to_file(file)?;
                }
            }
        }

        self.updates.clear();
        self.tracked.clear();

        // Update collection header...
        file.move_to(self.position)?;
        file.write_le_u64(self.o_next_collection)?;
        file.write_le_u8(self.record_type)?;
        file.write_le_u64(self.next_id)?;
        file.write_le_u64(record_cnt)?;
        file.write_le_u64(self.o_first_record)?;
        file.write_le_u64(self.o_last_record)?;

        Ok(())
    }
}
