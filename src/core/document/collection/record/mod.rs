pub mod delta;
pub mod location;

use std::fs::File;
use std::io;

use chrono::{DateTime, Utc};

use crate::utils::{
    extensions::file::{FileReadExtensions, FileWriteExtensions},
    file::FileHandle,
};

const HEADER_SIZE: u32 = 40;

#[derive(Clone)]
pub struct Record<T: RecordPayload> {
    pub o_prev: u64,
    pub o_next: u64,
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    pub item: T,
}

pub trait RecordPayload: FileHandle {
    fn record_type() -> u8;
    fn size(&self) -> u32;
}

impl<T: RecordPayload> Record<T> {
    pub fn new(o_prev: u64, id: u64, item: T) -> Self {
        Self {
            o_prev,
            o_next: 0,

            id,
            created_at: Utc::now(),
            last_modified_at: Utc::now(),
            item,
        }
    }

    pub fn size(&self) -> u32 {
        HEADER_SIZE + self.item.size()
    }
}

impl<T: RecordPayload> FileHandle for Record<T> {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        Ok(Record {
            o_prev: file.read_le_u64()?,
            o_next: file.read_le_u64()?,

            id: file.read_le_u64()?,
            created_at: file.read()?,
            last_modified_at: file.read()?,
            item: T::read_from_file(file)?,
        })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        file.write_le_u64(self.o_prev)?;
        file.write_le_u64(self.o_next)?;

        file.write_le_u64(self.id)?;
        file.write(&mut self.created_at)?;
        file.write(&mut self.last_modified_at)?;
        self.item.write_to_file(file)
    }
}
