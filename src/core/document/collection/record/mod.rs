pub mod delta;

use std::fs::File;
use std::io;

use crate::utils::extensions::file::{FileReadExtensions, FileWriteExtensions};

pub struct Record<T: RecordHandle> {
    pub o_prev: u64,
    pub o_next: u64,
    pub item: T,
}

pub trait RecordHandle: Sized {
    fn len(&self) -> u32;
    fn write(&mut self, file: &mut File) -> io::Result<()>;
    fn read(file: &mut File) -> io::Result<Self>;
}

impl<T: RecordHandle> Record<T> {
    pub fn new(o_prev: u64, item: T) -> Self {
        Self {
            o_prev,
            o_next: 0,
            item,
        }
    }
}

impl<T: RecordHandle> RecordHandle for Record<T> {
    fn len(&self) -> u32 {
        16 + self.item.len()
    }

    fn write(&mut self, file: &mut File) -> io::Result<()> {
        file.write_le_u64(self.o_prev)?;
        file.write_le_u64(self.o_next)?;
        self.item.write(file)
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
