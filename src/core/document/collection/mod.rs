mod account;
mod transaction;

pub use account::*;
pub use transaction::*;

use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};

use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};

pub struct Record<T: FileHandle> {
    pub o_prev: u64,
    pub o_next: u64,
    pub item: T,
}

pub struct Collection<T: FileHandle> {
    count: u32,
    for_append: Vec<Record<T>>,

    pub position: u64,
    pub o_first_record: u64,
    pub o_last_record: u64,
}

pub trait FileHandle: Sized {
    fn len(&self) -> u32;
    fn write(&mut self, file: &mut File, cursor: &mut u64) -> Result<()>;
    fn read(file: &mut File) -> Result<Self>;
}

impl<T: FileHandle> Record<T> {
    fn new(o_prev: u64, item: T) -> Self {
        Self {
            o_prev,
            o_next: 0,
            item,
        }
    }
}

impl<T: FileHandle> FileHandle for Record<T> {
    fn len(&self) -> u32 {
        16 + self.item.len()
    }

    fn write(&mut self, file: &mut File, _doc: &mut u64) -> Result<()> {
        file.write_all(&self.o_prev.to_le_bytes())?;
        file.write_all(&self.o_next.to_le_bytes())?;
        self.item.write(file, _doc)
    }

    fn read(file: &mut File) -> Result<Self> {
        let o_prev = file.read_le_u64()?;
        let o_next = file.read_le_u64()?;
        let item = T::read(file)?;

        Ok(Record {
            o_prev,
            o_next,
            item,
        })
    }
}

impl<T: FileHandle> Collection<T> {
    pub fn new(pos: u64) -> Self {
        Self {
            count: 0,
            for_append: Vec::new(),

            position: pos,
            o_first_record: 0,
            o_last_record: 0,
        }
    }

    pub fn index(&mut self, file: &mut File) -> Result<()> {
        file.move_to(self.position)?;

        self.count = file.read_le_u32()?;
        self.o_first_record = file.read_le_u64()?;
        self.o_last_record = file.read_le_u64()?;
        // +12 reserved bytes.

        Ok(())
    }

    pub fn get_all(&self, file: &mut File) -> Result<Vec<T>>
    where
        T: Clone,
    {
        let mut items: Vec<T> = vec![];

        let mut cursor = self.o_first_record;
        for _ in 0..self.count {
            file.move_to(cursor)?;

            let record = Record::<T>::read(file)?;
            items.push(record.item);

            cursor = record.o_next;
        }

        for trans in &self.for_append {
            items.push(trans.item.clone());
        }

        Ok(items)
    }

    pub fn add(&mut self, item: T) {
        let record = Record::new(self.o_last_record, item);
        self.for_append.push(record);
    }
}

impl<T: FileHandle> FileHandle for Collection<T> {
    fn len(&self) -> u32 {
        0
    }

    fn write(&mut self, file: &mut File, cursor: &mut u64) -> Result<()> {
        let append_len = self.for_append.len() as u32;
        let record_cnt = self.count + append_len;

        // Right records...
        if append_len > 0 {
            let mut records = self.for_append.iter_mut().peekable();

            let mut o_prev = self.o_last_record;
            let mut o_next = if self.o_last_record == 0 {
                // TODO: Revisit because this might not be correct.
                self.position + 20
            } else {
                *cursor
            };

            if self.o_last_record == 0 {
                self.o_first_record = o_next;
            } else {
                let offset = self.o_last_record + 8;
                file.seek(SeekFrom::Start(offset))?;
                file.write_all(&o_next.to_le_bytes())?;
            }

            file.seek(SeekFrom::Start(o_next))?;

            while let Some(record) = &mut records.next() {
                record.o_prev = o_prev;
                o_prev = o_next;
                match records.peek() {
                    Some(_) => {
                        o_next += record.len() as u64;
                        record.o_next = o_next;
                    }
                    None => {
                        record.o_next = 0;
                        self.o_last_record = o_next;
                    }
                }

                record.write(file, cursor)?;
            }
        }

        // Update cursor so next append knows
        // where to start.
        *cursor = file.stream_position()? + 1;

        // Update/write collection header...
        file.seek(SeekFrom::Start(self.position))?;
        file.write_all(&record_cnt.to_le_bytes())?;
        file.write_all(&self.o_first_record.to_le_bytes())?;
        file.write_all(&self.o_last_record.to_le_bytes())?;

        Ok(())
    }

    fn read(_: &mut File) -> Result<Self> {
        todo!()
    }
}
