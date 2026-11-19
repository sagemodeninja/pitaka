use std::fs::File;
use std::io::Result;

use super::Money;
use crate::utils::extensions::file::{FileReadExtensions, FileWriteExtensions};
use crate::utils::file::FileHandle;

impl FileHandle for Money {
    fn read_from_file(file: &mut File) -> Result<Self> {
        let minor_units = file.read_le_i64()?;
        Ok(Money::from_minor_units(minor_units))
    }

    fn write_to_file(&mut self, file: &mut File) -> Result<()> {
        let minor_units = self.minor_units();
        file.write_le_i64(minor_units)
    }
}
