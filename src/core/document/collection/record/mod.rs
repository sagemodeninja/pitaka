pub mod delta;

use std::fs::File;
use std::io;

use crate::{
    core::document::file::FileHandle,
    utils::extensions::file::{FileReadExtensions, FileWriteExtensions},
};

pub struct Record<T: RecordPayload> {
    pub o_prev: u64,
    pub o_next: u64,
    pub item: T,
}

pub trait RecordPayload: FileHandle {
    fn record_type() -> u8;
    fn size(&self) -> u32;
}

impl<T: RecordPayload> Record<T> {
    pub fn new(o_prev: u64, item: T) -> Self {
        Self {
            o_prev,
            o_next: 0,
            item,
        }
    }
}

impl<T: RecordPayload> FileHandle for Record<T> {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        Ok(Record {
            o_prev: file.read_le_u64()?,
            o_next: file.read_le_u64()?,
            item: T::read_from_file(file)?,
        })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        file.write_le_u64(self.o_prev)?;
        file.write_le_u64(self.o_next)?;
        self.item.write_to_file(file)
    }
}
