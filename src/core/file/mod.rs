pub mod collection;

use std::fs::{File, OpenOptions};
use std::io::Result;
use std::path::PathBuf;

use crate::core::file::collection::{Collection, FileHandle, Transaction};

pub struct PitakaFile {
    pub transactions: Collection<Transaction>,
}

impl PitakaFile {
    pub fn new() -> Self {
        Self {
            transactions: Collection::new(0),
        }
    }

    pub fn open(path: &PathBuf) -> Result<Self> {
        let transactions = read_transactions(path)?;
        Ok(PitakaFile { transactions })
    }

    pub fn save(&mut self, path: &PathBuf) -> Result<()> {
        write_transactions(path, &mut self.transactions)
    }
}

fn read_transactions(path: &PathBuf) -> Result<Collection<Transaction>> {
    let mut file = File::open(path)?;
    Collection::<Transaction>::read(&mut file)
}

fn write_transactions(path: &PathBuf, trans: &mut Collection<Transaction>) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).open(path)?;
    trans.write(&mut file)
}
