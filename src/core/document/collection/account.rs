use std::fs::File;
use std::io;

use crate::{
    core::document::collection::RecordHandle,
    utils::extensions::{
        file::{FileReadExtensions, FileWriteExtensions},
        string::StringExtensions,
    },
};

const ID_SIZE: u32 = 2;
const DP_LENGTH_SIZE: u32 = 4;

#[derive(Clone)]
pub struct Account {
    pub id: u16,
    pub display_name: String,
}

impl RecordHandle for Account {
    fn len(&self) -> u32 {
        ID_SIZE + DP_LENGTH_SIZE + self.display_name.len_u32()
    }

    fn write(&mut self, file: &mut File) -> io::Result<()> {
        file.write_le_u16(self.id)?;
        file.write_len_pre_str(&self.display_name)
    }

    fn read(file: &mut File) -> io::Result<Self> {
        Ok(Account {
            id: file.read_le_u16()?,
            display_name: file.read_len_pre_str()?,
        })
    }
}
