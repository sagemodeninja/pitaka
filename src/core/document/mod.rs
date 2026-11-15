pub mod collection;

use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use crate::core::document::collection::{
    Collection, FileHandle, Transaction,
};
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions,
};

const MINIMUM_FILE_SIZE: u64 = 16 + 24 + 32;
const MAGIC_NUMBER: [u8; 4] = *b"PTKA";
const CURRENT_FORMAT_VERSION: u16 = 1;

pub struct PitakaDocument {
    file: File,
    cursor: u64,
    pub transactions: Collection<Transaction>,
}

impl PitakaDocument {
    pub fn create(path: &PathBuf) -> Result<Self> {
        let file = open_file(path)?;

        let mut doc = Self {
            file,
            cursor: 0,
            transactions: Collection::new(0),
        };
        doc.save()?;

        Ok(doc)
    }

    pub fn open(path: &PathBuf) -> Result<Self> {
        let mut file = open_file(path)?;
        read_document(file)

        // let transactions = Collection::<Transaction>::read(&mut file)?;
        // let cursor = file.metadata()?.len();

        // Ok(PitakaDocument {
        //     file,
        //     cursor,
        //     transactions,
        // })
    }

    pub fn get_all_trans(&mut self) -> Result<Vec<Transaction>> {
        self.transactions.get_all(&mut self.file)
    }

    pub fn save(&mut self) -> Result<()> {
        self.transactions.write(&mut self.file, &mut self.cursor)
    }
}

fn open_file(path: &PathBuf) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
}

// SEE: /docs/develop/file_layout.md
fn read_document(mut file: File) -> Result<PitakaDocument> {
    let file_len = file.metadata()?.len();

    if file_len < MINIMUM_FILE_SIZE {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Document is too short: expected at least {MINIMUM_FILE_SIZE} bytes."
            ),
        ));
    }

    if file.read_u32_bytes()? != MAGIC_NUMBER {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Invalid PTKA magic number",
        ));
    }

    let format_ver = file.read_le_u16()?;
    if format_ver > CURRENT_FORMAT_VERSION {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Document is from a newer version. Current: {CURRENT_FORMAT_VERSION}, Found: {format_ver}."
            ),
        ));
    }

    file.move_for(10)?; // Reserved.

    // Collection discovery...
    let mut transactions = Collection::<Transaction>::new(0);

    loop {
        let c_type = file.read_le_u8()?;
        let c_next = file.read_le_u64()?;
        let c_pos = file.read_le_u64()?;

        match c_type {
            1 => transactions.position = c_pos,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Unsupported collection type: {c_type}."),
                ));
            }
        }

        if c_next > 0 {
            file.move_to(c_next)?;
        } else {
            break;
        }
    }

    transactions.read_self(&mut file)?;

    Ok(PitakaDocument {
        file: file,
        cursor: file_len,
        transactions,
    })
}
