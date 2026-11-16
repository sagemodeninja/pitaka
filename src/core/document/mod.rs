pub mod collection;

use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, Write};
use std::path::PathBuf;

use crate::core::document::collection::{Collection, FileHandle, Transaction};
use crate::utils::extensions::file::{FileCursorExtensions, FileReadExtensions};

const MAGIC_NUMBER: [u8; 4] = *b"PTKA";
const CURRENT_FORMAT_VERSION: u16 = 1;

const HEADER_SIZE: u64 = 16;
const POINTER_SIZE: u64 = 24;
const COLLECTION_SIZE: u64 = 32;
const MINIMUM_FILE_SIZE: u64 = HEADER_SIZE + POINTER_SIZE + COLLECTION_SIZE;

const INIT_COLLECTION_COUNT: u64 = 1;
const FIRST_COLLECTION_OFFSET: u64 = HEADER_SIZE + INIT_COLLECTION_COUNT * POINTER_SIZE;

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
        create_document(&mut doc)?;

        Ok(doc)
    }

    pub fn open(path: &PathBuf) -> Result<Self> {
        let file = open_file(path)?;
        read_document(file)
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

fn create_document(doc: &mut PitakaDocument) -> Result<()> {
    let file = &mut doc.file;

    file.move_to(0)?;

    // Header...
    file.write_all(&MAGIC_NUMBER)?;
    file.write_all(&CURRENT_FORMAT_VERSION.to_le_bytes())?;
    file.move_for(10)?; // Reserved

    // Pointers...
    for c_type in 1..=INIT_COLLECTION_COUNT {
        let collection = match c_type {
            1 => &mut doc.transactions,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Unsupported collection type: {c_type}."),
                ));
            }
        };

        let o_p_next = if c_type < INIT_COLLECTION_COUNT {
            HEADER_SIZE + c_type * POINTER_SIZE
        } else {
            0
        };

        let c_pos = FIRST_COLLECTION_OFFSET + (c_type - 1) * COLLECTION_SIZE;
        collection.position = c_pos;

        // Pointer...
        file.write_all(&[c_type as u8])?;
        file.write_all(&o_p_next.to_le_bytes())?;
        file.write_all(&c_pos.to_le_bytes())?;
        file.move_for(7)?; // Reserved
    }

    // Write last byte which fills gaps with 0.
    file.move_to(FIRST_COLLECTION_OFFSET + INIT_COLLECTION_COUNT * COLLECTION_SIZE)?;
    file.write_all(&[0u8])?;

    doc.cursor = file.stream_position()?;
    Ok(())
}

// SEE: /docs/develop/file_layout.md
fn read_document(mut file: File) -> Result<PitakaDocument> {
    let file_len = file.metadata()?.len();

    file.move_to(0)?;

    if file_len < MINIMUM_FILE_SIZE {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Document is too short: expected at least {MINIMUM_FILE_SIZE} bytes."),
        ));
    }

    if file.read_4_bytes()? != MAGIC_NUMBER {
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

    // Index collections...
    transactions.index(&mut file)?;

    file.move_to(0)?;

    Ok(PitakaDocument {
        file: file,
        cursor: file_len,
        transactions,
    })
}
