mod account;
mod directory;
mod header;
mod index;
mod managed;
mod query;
mod record;
mod root;
mod transaction;

pub use account::*;
pub use header::*;
pub use managed::*;
pub use record::*;
pub use root::*;
pub use transaction::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self};
use std::os::unix::fs::MetadataExt;

use crate::core::vault::collection::delta::RecordDelta::{self, Create, Delete};
use crate::core::vault::collection::directory::RecordDirectory;
use crate::core::vault::collection::index::{CollectionIndexBuilder, IndexValue};
use crate::core::vault::collection::location::RecordLocation::{self, InMemory, OnDisk};
use crate::core::vault::collection::query::CollectionQueryBuilder;
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};
use crate::utils::file::FileHandle;

const COLLECTION_SIZE: u64 = 40;

pub struct Collection<T: RecordPayload> {
    header: CollectionHeader,

    directory: RecordDirectory,
    indexes: CollectionIndexBuilder<T>,
    updates: Vec<(u64, RecordDelta)>,

    tracked_updates: HashMap<u64, Record<T>>,
}

pub trait CollectionType: RecordPayload {
    fn c_type() -> u8;
}

pub trait CollectionHandle: FileHandle {
    fn header(&self) -> &CollectionHeader;
}

impl<T: RecordPayload> Collection<T> {
    pub fn new(header: CollectionHeader) -> Self {
        Self {
            header,

            directory: RecordDirectory::new(),
            indexes: CollectionIndexBuilder::new(),
            updates: Vec::new(),

            tracked_updates: HashMap::new(),
        }
    }

    pub fn add_index<'a>(
        &mut self,
        key: &str,
        eval: impl Fn(&T) -> IndexValue + 'static,
    ) -> &mut CollectionIndexBuilder<T> {
        self.indexes.add_index(key, eval)
    }

    pub fn build_indexes(&mut self, file: &mut File) -> io::Result<()> {
        let header = &self.header;
        let mut o_r_next = header.o_first_record;

        for _ in 0..header.record_count {
            file.move_to(o_r_next)?;

            let record = Record::<T>::read_from_file(file)?;

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
                    Box::new(self.directory.iter())
                } else {
                    let ids = plan
                        .indexed_filters
                        .iter()
                        .next()
                        .and_then(|(key, value)| self.indexes.get(key)?.get(value))
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);

                    Box::new(ids.iter().filter_map(move |id| self.directory.get(id)))
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
                        if let Some(item) = self.tracked_updates.get(id) {
                            records.push(item.clone());
                        }
                    }
                }
            }

            Ok(records)
        })
    }

    pub fn add(&mut self, item: T) {
        let header = &mut self.header;

        let id = header.next_record_id;
        let record = Record::new(header.o_last_record, id, item);

        self.directory.insert(id, InMemory);
        self.indexes.insert(&record);
        self.updates.push((id, RecordDelta::Create(id)));
        self.tracked_updates.insert(id, record);

        header.next_record_id += 1;
    }

    pub fn delete(&mut self, id: u64) {
        if let Some((index, (_, location))) = self.directory.get_with_index(&id) {
            match location {
                OnDisk(offset) => {
                    self.updates.push((id, Delete(*offset)));
                }
                InMemory => {
                    self.updates.retain(|(target_id, _)| *target_id != id);
                }
            }

            self.tracked_updates.remove(&id);
            self.directory.remove(index);
        }
    }
}

impl<T: RecordPayload> FileHandle for Collection<T> {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        Ok(Self {
            header: CollectionHeader::read_from_file(file)?,

            directory: RecordDirectory::new(),
            indexes: CollectionIndexBuilder::new(),
            updates: Vec::new(),

            tracked_updates: HashMap::new(),
        })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let header = &mut self.header;

        let mut for_append: Vec<u64> = vec![];
        let mut for_remove: Vec<u64> = vec![];

        for (_, delta) in self.updates.iter() {
            match delta {
                Create(id) => for_append.push(*id),
                Delete(offset) => for_remove.push(*offset),
                _ => {}
            }
        }

        let append_len = for_append.len() as u64;
        let remove_len = for_remove.len() as u64;
        let record_cnt = header.record_count + append_len - remove_len;

        // Perform deletions...
        for d_offset in for_remove {
            file.move_to(d_offset)?;

            let o_prev = file.read_le_u64()?;
            let o_next = file.read_le_u64()?;

            if o_prev != 0 {
                file.move_to(o_prev + 8)?;
                file.write_le_u64(o_next)?;
            } else {
                header.o_first_record = o_next;
            }

            if o_next != 0 {
                file.move_to(o_next)?;
                file.write_le_u64(o_prev)?;
            } else {
                header.o_last_record = o_prev;
            }

            // TODO: What happens to the current record?
        }

        // Append new records...
        if append_len > 0 {
            let mut o_prev = header.o_last_record;
            let mut o_next = file.metadata()?.size();

            if header.record_count - remove_len == 0 {
                header.o_first_record = o_next;
            } else {
                file.move_to(header.o_last_record + 8)?;
                file.write_le_u64(o_next)?;
            }

            // Append new records...
            for index in 0..append_len {
                let is_last = index == append_len - 1;

                if is_last {
                    header.o_last_record = o_next;
                }

                let id = for_append[index as usize];
                if let Some(record) = self.tracked_updates.get_mut(&id) {
                    let o_curr = o_next;

                    file.move_to(o_curr)?;

                    record.o_prev = o_prev;

                    o_prev = o_next;
                    o_next = if !is_last {
                        o_next + record.size() as u64
                    } else {
                        0
                    };

                    record.o_next = o_next;
                    record.write_to_file(file)?;

                    self.directory.update(id, OnDisk(o_curr))?;
                }
            }
        }

        file.move_to(header.position)?;
        header.record_count = record_cnt;
        header.write_to_file(file)?;

        self.updates.clear();
        self.tracked_updates.clear();

        Ok(())
    }
}

impl<T: RecordPayload> CollectionHandle for Collection<T> {
    fn header(&self) -> &CollectionHeader {
        &self.header
    }
}
