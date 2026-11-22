use std::{
    fs::File,
    io::{self, Write},
};

use thiserror::Error;

use crate::utils::{
    extensions::file::{FileCursorExtensions, FileReadExtensions, FileWriteExtensions},
    file::FileHandle,
};

const MAGIC_NUMBER: [u8; 4] = *b"PTKA";
const CURRENT_FORMAT_VERSION: u16 = 1;

pub struct PitakaVaultHeader {
    pub magic_bytes: [u8; 4],
    pub format_version: u16,
    pub display_name: String,
    // +6 reserved bytes.
}

impl PitakaVaultHeader {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            magic_bytes: MAGIC_NUMBER,
            format_version: CURRENT_FORMAT_VERSION,
            display_name: name.into(),
        }
    }

    pub fn validate(&self) -> Result<(), PitakaVaultHeaderError> {
        if self.magic_bytes != MAGIC_NUMBER {
            return Err(PitakaVaultHeaderError::Validation(
                "Invalid PTKA magic number".into(),
            ));
        }

        let version = self.format_version;
        if version <= 0 || version > CURRENT_FORMAT_VERSION {
            return Err(PitakaVaultHeaderError::Validation(format!(
                "Unsupported vault version. Current: {CURRENT_FORMAT_VERSION}, Found: {version}.",
            )));
        }

        Ok(())
    }
}

impl FileHandle for PitakaVaultHeader {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        file.move_to(0)?;

        Ok(Self {
            magic_bytes: file.read_4_bytes()?,
            format_version: file.read_le_u16()?,
            display_name: file.read_len_pre_str()?,
        })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        file.move_to(0)?;

        file.write_all(&self.magic_bytes)?;
        file.write_le_u16(self.format_version)?;
        file.write_len_pre_str(&self.display_name)?;

        let dn_len = self.display_name.len();
        if dn_len < 64 {
            file.write_zeros(64 - dn_len)?;
        }

        // Reserved...
        file.write_zeros(6)
    }
}

// Error...

#[derive(Error, Debug)]
pub enum PitakaVaultHeaderError {
    #[error("{0}")]
    Validation(String),
}

impl From<PitakaVaultHeaderError> for io::Error {
    fn from(error: PitakaVaultHeaderError) -> Self {
        Self::new(io::ErrorKind::InvalidData, error)
    }
}
