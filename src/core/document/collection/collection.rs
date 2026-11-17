use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom, Write};

use crate::core::document::collection::Record;
use crate::core::document::collection::delta::{RecordDelta, RecordDeltaType};
use crate::core::document::collection::query::CollectionQueryBuilder;
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};

const COLLECTION_SIZE: u64 = 48;
const FIRST_COLLECTION_OFFSET: u64 = 16;
const INIT_COLLECTION_COUNT: u64 = 2;

pub struct Collection<T: FileHandle> {
    next_id: u64,
    updates: Vec<RecordDelta>,
    tracked_records: HashMap<u64, Record<T>>,

    pub position: u64,
    pub record_count: u64,
    pub o_first_record: u64,
    pub o_last_record: u64,
}

pub trait FileHandle: Sized {
    fn len(&self) -> u32;
    fn write(&mut self, file: &mut File, cursor: &mut u64) -> io::Result<()>;
    fn read(file: &mut File) -> io::Result<Self>;
}

impl<T: FileHandle> Collection<T> {
    pub fn empty(position: u64) -> Self {
        Self {
            next_id: 1,
            record_count: 0,
            updates: Vec::new(),
            tracked_records: HashMap::new(),

            position,
            o_first_record: 0,
            o_last_record: 0,
        }
    }

    pub fn create(c_type: u64, file: &mut File) -> io::Result<Self> {
        let o_c_curr = FIRST_COLLECTION_OFFSET + (c_type - 1) * COLLECTION_SIZE;
        let o_c_next = if c_type < INIT_COLLECTION_COUNT {
            FIRST_COLLECTION_OFFSET + c_type * COLLECTION_SIZE
        } else {
            0
        };

        file.move_to(o_c_curr)?;
        file.write_le_u64(o_c_next)?;
        file.write_le_u8(c_type as u8)?;
        file.write_le_u64(1)?;
        file.write_zeros(31)?; // Remaining bytes.

        Ok(Self::empty(o_c_curr))
    }

    pub fn read(&mut self, position: u64, file: &mut File) -> io::Result<()> {
        self.next_id = file.read_le_u64()?;

        self.position = position;
        self.record_count = file.read_le_u64()?;
        self.o_first_record = file.read_le_u64()?;
        self.o_last_record = file.read_le_u64()?;
        // +7 reserved bytes.

        Ok(())
    }

    pub fn get_all(&self, file: &mut File) -> io::Result<Vec<T>>
    where
        T: Clone,
    {
        let mut items: Vec<T> = vec![];

        let mut cursor = self.o_first_record;
        for _ in 0..self.record_count {
            file.move_to(cursor)?;

            let record = Record::<T>::read(file)?;
            items.push(record.item);

            cursor = record.o_next;
        }

        // for item in &self.for_append {
        //     items.push(item.item.clone());
        // }

        Ok(items)
    }

    pub fn get_by_offsets<'a>(
        &'a self,
        offsets: &'a Vec<u64>,
        file: &'a mut File,
    ) -> CollectionQueryBuilder<'a, T>
    where
        T: Clone,
    {
        // TODO: A way for for_append to participate on the query.

        CollectionQueryBuilder::new(move |plan| {
            let mut items: Vec<T> = vec![];

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
                items.push(T::read(file)?);
            }

            // for item in &self.for_append {
            //     items.push(item.item.clone());
            // }

            Ok(items)
        })
    }

    pub fn add(&mut self, ctor: impl FnOnce(u64) -> T) {
        let id = self.next_id;
        let item = ctor(id);

        let record = Record::new(self.o_last_record, item);
        self.tracked_records.insert(id, record);

        let delta = RecordDelta::create(id);
        self.updates.push(delta);

        self.next_id += 1;
    }
}

impl<T: FileHandle> FileHandle for Collection<T> {
    fn len(&self) -> u32 {
        0
    }

    fn write(&mut self, file: &mut File, cursor: &mut u64) -> io::Result<()> {
        let for_append: Vec<_> = self
            .updates
            .iter()
            .filter(|delta| matches!(delta.delta_type, RecordDeltaType::Create))
            .collect();

        let append_len = for_append.len() as u64;
        let record_cnt = self.record_count + append_len;

        if append_len > 0 {
            let mut o_prev = self.o_last_record;
            let mut o_next = *cursor;

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
                        o_next + record.len() as u64
                    } else {
                        0
                    };

                    record.o_next = o_next;
                    record.write(file, cursor)?;
                }
            }
        }

        // Update cursor so next append knows
        // where to start.
        *cursor = file.stream_position()?;

        // Update collection header...
        file.move_to(self.position + 9)?;
        file.write_le_u64(self.next_id)?;
        file.write_le_u64(record_cnt)?;
        file.write_le_u64(self.o_first_record)?;
        file.write_le_u64(self.o_last_record)?;

        Ok(())
    }

    fn read(_: &mut File) -> io::Result<Self> {
        todo!()
    }
}
