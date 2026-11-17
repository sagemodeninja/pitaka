pub mod collection;

use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Error, ErrorKind, Seek, Write};
use std::path::PathBuf;

use crate::core::document::collection::{Account, Collection, Transaction};
use crate::utils::extensions::file::{
    FileCursorExtensions, FileReadExtensions, FileWriteExtensions,
};

const MAGIC_NUMBER: [u8; 4] = *b"PTKA";
const CURRENT_FORMAT_VERSION: u16 = 1;

const HEADER_SIZE: u64 = 16;
const COLLECTION_SIZE: u64 = 48;
const MINIMUM_FILE_SIZE: u64 = HEADER_SIZE + COLLECTION_SIZE;

pub struct PitakaDocument {
    file: File,
    cursor: u64,

    pub accounts: Collection<Account>,
    pub transactions: Collection<Transaction>,
}

impl PitakaDocument {
    // SEE: /docs/develop/file_layout.md
    pub fn create(path: &PathBuf) -> io::Result<Self> {
        let mut file = open_file(path, true)?;

        // Header...
        file.move_to(0)?;
        file.write_all(&MAGIC_NUMBER)?;
        file.write_le_u16(CURRENT_FORMAT_VERSION)?;
        file.write_zeros(10)?; // Reserved...

        // Collections...
        let accounts = Collection::create(1, &mut file)?;
        let transactions = Collection::create(2, &mut file)?;

        let cursor = file.stream_position()?;
        file.move_to(0)?;

        Ok(PitakaDocument {
            file,
            cursor,
            accounts,
            transactions,
        })
    }

    pub fn open(path: &PathBuf) -> io::Result<Self> {
        let file = open_file(path, false)?;
        read_document(file)
    }

    pub fn get_all_accts(&mut self) -> io::Result<Vec<Account>> {
        self.accounts.get_all(&mut self.file)
    }

    pub fn get_acct_trans(&mut self, id: u16) -> io::Result<Vec<Transaction>> {
        self.transactions
            // .get_by_offsets(offsets, &mut self.file)
            .query(&mut self.file)
            .filter_indexed("acct_id", id.into())
            .reverse()
            .collect()
    }

    pub fn save(&mut self) -> io::Result<()> {
        self.accounts.write(&mut self.file, &mut self.cursor)?;
        self.transactions.write(&mut self.file, &mut self.cursor)?;
        Ok(())
    }
}

fn open_file(path: &PathBuf, truncate: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(path)
}

// SEE: /docs/develop/file_layout.md
fn read_document(mut file: File) -> io::Result<PitakaDocument> {
    let file_len = file.metadata()?.len();

    // Header...
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
    let mut accounts = Collection::<Account>::empty(0);
    let mut transactions = Collection::<Transaction>::empty(0);

    transactions.index("acct_id", |trans| trans.account_id.into());

    loop {
        let o_c_curr = file.stream_position()?;
        let o_c_next = file.read_le_u64()?;
        let c_type = file.read_le_u8()?;

        match c_type {
            1 => accounts.read(o_c_curr, &mut file)?,
            2 => transactions.read(o_c_curr, &mut file)?,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Unsupported collection type: {c_type}."),
                ));
            }
        }

        if o_c_next > 0 {
            file.move_to(o_c_next)?;
        } else {
            break;
        }
    }

    file.move_to(0)?;

    Ok(PitakaDocument {
        file,
        cursor: file_len,
        accounts,
        transactions,
    })
}
