mod account;
mod index;
mod query;
mod record;
mod transaction;

pub use account::*;
pub use record::*;
pub use transaction::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self};
use std::os::unix::fs::MetadataExt;

use crate::core::document::collection::delta::{RecordDelta, RecordDeltaType};
use crate::core::document::collection::index::{CollectionIndexBuilder, IndexValue};
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

    indexes: CollectionIndexBuilder<T>,
    updates: Vec<RecordDelta>,
    tracked_records: HashMap<u64, Record<T>>,
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

            indexes: CollectionIndexBuilder::new(),
            updates: Vec::new(),
            tracked_records: HashMap::new(),
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
    pub fn read(&mut self, position: u64, file: &mut File) -> io::Result<()> {
        self.next_id = file.read_le_u64()?;

        self.position = position;
        self.record_count = file.read_le_u64()?;
        self.o_first_record = file.read_le_u64()?;
        self.o_last_record = file.read_le_u64()?;
        // +7 reserved bytes.

        Ok(())
    }

    pub fn index<'a>(
        &mut self,
        key: &str,
        eval: impl Fn(&T) -> IndexValue + 'static,
    ) -> &mut CollectionIndexBuilder<T> {
        self.indexes.index(key, eval)
    }

    pub fn build_index(&mut self, file: &mut File) -> io::Result<()> {
        self.indexes
            .build(self.record_count, self.o_first_record, file)
    }

    pub fn get_all(&self, file: &mut File) -> io::Result<Vec<Record<T>>>
    where
        T: Clone,
    {
        let mut records = Vec::new();

        let mut cursor = self.o_first_record;
        for _ in 0..self.record_count {
            file.move_to(cursor)?;

            let record = Record::<T>::read_from_file(file)?;
            cursor = record.o_next;

            records.push(record);
        }

        // for item in &self.for_append {
        //     items.push(item.item.clone());
        // }

        Ok(records)
    }

    pub fn add(&mut self, item: T) {
        let id = self.next_id;

        let record = Record::new(self.o_last_record, id, item);
        self.tracked_records.insert(id, record);

        let delta = RecordDelta::create(id);
        self.updates.push(delta);

        self.next_id += 1;
    }

    pub fn query<'a>(&'a self, file: &'a mut File) -> CollectionQueryBuilder<'a, T>
    where
        T: Clone,
    {
        // TODO: A way for for_append to participate on the query.
        CollectionQueryBuilder::new(move |plan| {
            let mut items: Vec<T> = vec![];

            let mut offsets: Option<&Vec<u64>> = None;
            for (key, value) in plan.indexed_filters {
                if let Some(index) = self.indexes.get(&key) {
                    offsets = index.get(&value);
                    break;
                }
            }

            let offsets: &[u64] = offsets.map(Vec::as_slice).unwrap_or(&[]);

            let iter: Box<dyn Iterator<Item = &u64> + '_> = if plan.is_reversed {
                Box::new(offsets.iter().rev())
            } else {
                Box::new(offsets.iter())
            };

            let take = match plan.take {
                Some(take) => take as usize,
                None => usize::MAX,
            };

            for &offset in iter.skip(plan.skip as usize).take(take) {
                file.move_to(offset + 16)?;
                items.push(T::read_from_file(file)?);
            }

            // for item in &self.for_append {
            //     items.push(item.item.clone());
            // }

            Ok(items)
        })
    }
}

impl<T: RecordPayload> FileHandle for Collection<T> {
    fn read_from_file(_: &mut File) -> io::Result<Self> {
        todo!()
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let for_append: Vec<_> = self
            .updates
            .iter()
            .filter(|delta| matches!(delta.delta_type, RecordDeltaType::Create))
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
                println!("Index: {index}. Pos: {o_next}");
                let is_last = index == append_len - 1;

                if is_last {
                    self.o_last_record = o_next;
                }

                let delta = for_append[index as usize];
                if let Some(record) = self.tracked_records.get_mut(&delta.record_id) {
                    file.move_to(o_next)?;

                    record.o_prev = o_prev;

                    o_prev = o_next;
                    o_next = if !is_last {
                        o_next + record.item.size() as u64
                    } else {
                        0
                    };

                    record.o_next = o_next;
                    record.write_to_file(file)?;
                }
            }
        }

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
