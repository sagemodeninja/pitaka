pub mod collection;

use std::fs::{File, OpenOptions};
use std::io::Result;
use std::path::PathBuf;

use crate::core::document::collection::{Collection, FileHandle, Transaction};

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

        let transactions = Collection::<Transaction>::read(&mut file)?;
        let cursor = file.metadata()?.len() + 1;

        Ok(PitakaDocument {
            file,
            cursor,
            transactions,
        })
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
