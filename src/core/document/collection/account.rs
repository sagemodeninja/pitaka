use std::fs::File;
use std::io;

use crate::{
    core::document::collection::RecordPayload,
    utils::{
        extensions::{
            file::{FileReadExtensions, FileWriteExtensions},
            string::StringExtensions,
        },
        file::FileHandle,
    },
};

#[derive(Clone)]
pub struct Account {
    pub display_name: String,
}

impl RecordPayload for Account {
    fn record_type() -> u8 {
        1
    }

    fn size(&self) -> u32 {
        self.display_name.len_u32()
    }
}

impl FileHandle for Account {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        Ok(Account {
            display_name: file.read_len_pre_str()?,
        })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        file.write_len_pre_str(&self.display_name)
    }
}
