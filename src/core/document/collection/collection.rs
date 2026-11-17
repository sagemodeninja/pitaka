use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom, Write};

use crate::core::document::collection::query::CollectionQueryBuilder;
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};

const COLLECTION_SIZE: u64 = 48;
const FIRST_COLLECTION_OFFSET: u64 = 16;
const INIT_COLLECTION_COUNT: u64 = 2;

pub struct Record<T: FileHandle> {
    pub o_prev: u64,
    pub o_next: u64,
    pub item: T,
}

pub struct Collection<T: FileHandle> {
    next_id: u64,
    for_append: Vec<Record<T>>,

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

    fn write(&mut self, file: &mut File, _doc: &mut u64) -> io::Result<()> {
        file.write_all(&self.o_prev.to_le_bytes())?;
        file.write_all(&self.o_next.to_le_bytes())?;
        self.item.write(file, _doc)
    }

    fn read(file: &mut File) -> io::Result<Self> {
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
    pub fn empty(position: u64) -> Self {
        Self {
            next_id: 1,
            record_count: 0,
            for_append: Vec::new(),

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

        for item in &self.for_append {
            items.push(item.item.clone());
        }

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
        CollectionQueryBuilder::new(move |plan| {
            let mut items: Vec<T> = vec![];

            for &offset in offsets
                .iter()
                .skip(plan.skip as usize)
                .take(plan.take as usize)
            {
                file.move_to(offset + 16)?;
                items.push(T::read(file)?);
            }

            for item in &self.for_append {
                items.push(item.item.clone());
            }

            Ok(items)
        })
    }

    pub fn add(&mut self, ctor: impl FnOnce(u64) -> T) {
        let item = ctor(self.next_id);
        let record = Record::new(self.o_last_record, item);

        self.next_id += 1;
        self.for_append.push(record);
    }
}

impl<T: FileHandle> FileHandle for Collection<T> {
    fn len(&self) -> u32 {
        0
    }

    fn write(&mut self, file: &mut File, cursor: &mut u64) -> io::Result<()> {
        let append_len = self.for_append.len() as u64;
        let record_cnt = self.record_count + append_len;

        // Right records...
        if append_len > 0 {
            let mut records = self.for_append.iter_mut().peekable();

            let mut o_prev = self.o_last_record;
            let mut o_next = *cursor;

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

        self.for_append.clear();
        // Update cursor so next append knows
        // where to start.
        *cursor = file.stream_position()? + 1;

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
