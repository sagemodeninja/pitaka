pub mod delta;

use std::fs::File;
use std::io;

use super::FileHandle;
use crate::utils::extensions::file::{FileReadExtensions, FileWriteExtensions};

pub struct Record<T: FileHandle> {
    pub o_prev: u64,
    pub o_next: u64,
    pub item: T,
}

impl<T: FileHandle> Record<T> {
    pub fn new(o_prev: u64, item: T) -> Self {
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
        file.write_le_u64(self.o_prev)?;
        file.write_le_u64(self.o_next)?;
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
