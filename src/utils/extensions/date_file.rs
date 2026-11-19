use std::fs::File;
use std::io::{Error, ErrorKind, Result};

use chrono::{DateTime, Utc};

use crate::utils::extensions::file::{FileReadExtensions, FileWriteExtensions};
use crate::utils::file::FileHandle;

impl FileHandle for DateTime<Utc> {
    fn read_from_file(file: &mut File) -> Result<Self> {
        let millis = file.read_le_i64()?;
        DateTime::from_timestamp_millis(millis).ok_or(Error::new(
            ErrorKind::InvalidData,
            "Unix timestamp in milliseconds is out of range",
        ))
    }

    fn write_to_file(&mut self, file: &mut File) -> Result<()> {
        let millis = self.timestamp_millis();
        file.write_le_i64(millis)
    }
}
