mod account;
mod any;
mod directory;
mod error;
mod header;
mod index;
mod query;
mod record;
mod root;
mod transaction;

pub use account::*;
pub use any::*;
pub use error::*;
pub use header::*;
pub use record::*;
pub use root::*;
pub use transaction::*;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self};
use std::os::unix::fs::MetadataExt;

use crate::core::vault::collection::delta::RecordDelta::{self, Create, Delete, Update};
use crate::core::vault::collection::directory::RecordDirectory;
use crate::core::vault::collection::index::{CollectionIndexBuilder, IndexValue};
use crate::core::vault::collection::location::RecordLocation::{self, InMemory, OnDisk};
use crate::core::vault::collection::query::CollectionQueryBuilder;
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};
use crate::utils::file::FileHandle;

pub struct Collection<T: RecordPayload> {
    header: CollectionHeader,
    directory: RecordDirectory,
    indexes: CollectionIndexBuilder<T>,
    updates: BTreeMap<u64, RecordDelta<T>>,
}

pub trait CollectionType: RecordPayload {
    fn c_type() -> u8;
}

impl<T: RecordPayload> Collection<T> {
    pub fn new(header: CollectionHeader) -> Self {
        Self {
            header,
            directory: RecordDirectory::new(),
            indexes: CollectionIndexBuilder::new(),
            updates: BTreeMap::new(),
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
            self.indexes.insert(record.id, &record.item);

            o_r_next = record.o_next;
        }

        Ok(())
    }

    pub fn query<'a>(&'a self, file: &'a mut File) -> CollectionQueryBuilder<'a, T>
    where
        T: Clone,
    {
        CollectionQueryBuilder::new(move |plan| {
            let mut results = Vec::<RecordSlice<T>>::new();

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
                        let record = Record::<T>::read_from_file(file)?;
                        results.push(record.into());
                    }
                    InMemory => {
                        if let Some(delta) = self.updates.get(id) {
                            let item = match delta {
                                Create(id, item) | Update(id, _, item) => {
                                    Some(RecordSlice::new(*id, item.clone()))
                                }
                                _ => None,
                            };

                            if let Some(item) = item {
                                results.push(item);
                            }
                        }
                    }
                }
            }

            Ok(results)
        })
    }

    pub fn add(&mut self, item: T) {
        let header = &mut self.header;
        let id = header.next_record_id;

        self.directory.insert(id, InMemory);
        self.indexes.insert(id, &item);
        self.updates.insert(id, RecordDelta::Create(id, item));

        header.next_record_id += 1;
    }

    pub fn update(&mut self, id: u64, item: T) -> Result<(), CollectionError> {
        match self.directory.get(&id) {
            Some((_, location)) => {
                match location {
                    OnDisk(offset) => {
                        let offset = *offset;
                        self.directory.update(id, InMemory)?;
                        self.updates
                            .insert(id, RecordDelta::Update(id, offset, item));
                    }
                    InMemory => {
                        if let Some(delta) = self.updates.get(&id) {
                            match delta {
                                Create(..) => {
                                    // Just change what's stored...
                                    self.updates.insert(id, RecordDelta::Create(id, item));
                                }
                                Update(_, offset, _) => {
                                    let delta = RecordDelta::Update(id, *offset, item);
                                    self.updates.insert(id, delta);
                                }
                                _ => {
                                    return Err(CollectionError::InvalidOperation(
                                        "Cannot update a deleted record.".into(),
                                    ));
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
            None => Err(CollectionError::RecordNotFound(id)),
        }
    }

    pub fn delete(&mut self, id: u64) -> Result<(), CollectionError> {
        match self.directory.get_with_index(&id) {
            Some((index, (_, location))) => {
                match location {
                    OnDisk(offset) => {
                        self.updates.insert(id, Delete(*offset));
                        self.directory.remove(index);
                    }
                    InMemory => {
                        if let Some(delta) = self.updates.get(&id) {
                            match delta {
                                Create(..) => {
                                    self.updates.remove(&id);
                                    self.directory.remove(index);
                                }
                                Update(_, offset, _) => {
                                    self.updates.insert(id, Delete(*offset));
                                    self.directory.remove(index);
                                }
                                Delete(..) => {}
                            }
                        }
                    }
                }
                Ok(())
            }
            None => Err(CollectionError::RecordNotFound(id)),
        }
    }
}

impl<T: RecordPayload> FileHandle for Collection<T> {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        Ok(Self {
            header: CollectionHeader::read_from_file(file)?,
            directory: RecordDirectory::new(),
            indexes: CollectionIndexBuilder::new(),
            updates: BTreeMap::new(),
        })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let header = &mut self.header;

        let mut for_append: Vec<&RecordDelta<T>> = vec![];
        let mut for_update: Vec<&RecordDelta<T>> = vec![];
        let mut for_remove: Vec<u64> = vec![];

        for (_, delta) in &self.updates {
            match delta {
                Create(..) => for_append.push(delta),
                Update(..) => for_update.push(delta),
                Delete(offset) => for_remove.push(*offset),
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
            let mut o_curr = header.o_last_record;
            let mut o_next = file.metadata()?.size();

            if header.record_count - remove_len == 0 {
                header.o_first_record = o_next;
            } else {
                file.move_to(header.o_last_record + 8)?;
                file.write_le_u64(o_next)?;
            }

            // Append new records...
            for index in 0..append_len {
                if let Create(id, item) = for_append.remove(index as usize) {
                    let mut record = Record::<T>::new(o_curr, *id, item.clone());

                    o_curr = o_next;
                    o_next = if index < append_len {
                        o_next + record.size() as u64
                    } else {
                        0
                    };

                    record.o_next = o_next;

                    file.move_to(o_curr)?;
                    record.write_to_file(file)?;

                    self.directory.update(*id, OnDisk(o_curr))?;
                }
            }

            header.o_last_record = o_curr;
        }

        for delta in for_update {
            if let Update(id, offset, item) = delta {
                file.move_to(*offset)?;

                let o_prev = file.read_le_u64()?;
                let o_next = file.read_le_u64()?;
                let o_curr = file.metadata()?.size();

                let mut record = Record::<T>::new(o_prev, *id, item.clone());
                record.o_next = o_next;

                file.move_to(o_curr)?;
                record.write_to_file(file)?;

                if o_prev != 0 {
                    file.move_to(o_prev + 8)?;
                    file.write_le_u64(o_curr)?;
                } else {
                    header.o_first_record = o_curr;
                }

                if o_next != 0 {
                    file.move_to(o_next)?;
                    file.write_le_u64(o_curr)?;
                } else {
                    header.o_last_record = o_curr;
                }

                self.directory.update(*id, OnDisk(o_curr))?;
            }
        }

        file.move_to(header.position)?;
        header.record_count = record_cnt;
        header.write_to_file(file)?;

        self.updates.clear();

        Ok(())
    }
}
