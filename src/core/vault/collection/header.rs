use std::{
    fs::File,
    io::{self, Seek},
};

use crate::utils::{extensions::file::*, file::FileHandle};

const RESERVED_SIZE: usize = 7;

#[derive(Clone, Copy)]
pub struct CollectionHeader {
    pub c_type: u8,
    pub record_count: u64,
    pub next_record_id: u64,
    pub o_first_record: u64,
    pub o_last_record: u64,
    // +7 reserved bytes.
    pub position: u64,
}

impl CollectionHeader {
    pub fn new(c_type: u8, position: u64) -> Self {
        Self {
            c_type,
            record_count: 0,
            next_record_id: 1,
            o_first_record: 0,
            o_last_record: 0,
            position,
        }
    }
}

impl FileHandle for CollectionHeader {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        let position = file.stream_position()?;

        let header = Self {
            c_type: file.read_le_u8()?,
            record_count: file.read_le_u64()?,
            next_record_id: file.read_le_u64()?,
            o_first_record: file.read_le_u64()?,
            o_last_record: file.read_le_u64()?,
            position,
        };

        Ok(header)
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        if self.position == 0 {
            self.position = file.stream_position()?;
        }

        file.write_le_u8(self.c_type)?;
        file.write_le_u64(self.record_count)?;
        file.write_le_u64(self.next_record_id)?;
        file.write_le_u64(self.o_first_record)?;
        file.write_le_u64(self.o_last_record)?;
        file.write_zeros(RESERVED_SIZE)
    }
}
