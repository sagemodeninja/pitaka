pub mod collection;
mod header;

use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;

use crate::utils::file::FileHandle;
use collection::{Account, Collection, RootCollection, Transaction};
use collection::{RecordSlice, filter};
use header::PitakaVaultHeader;

pub struct PitakaVault {
    file: File,
    header: PitakaVaultHeader,
    collections: RootCollection,
}

impl PitakaVault {
    pub fn create(name: impl Into<String>, path: &PathBuf) -> io::Result<Self> {
        let mut file = open_file(path, true)?;

        let mut header = PitakaVaultHeader::new(name);

        let mut collections = RootCollection::create();

        // Write to file...
        header.write_to_file(&mut file)?;
        collections.write_to_file(&mut file)?;
        collections.init(&mut file)?;

        Ok(Self {
            file,
            header,
            collections,
        })
    }

    pub fn open(path: &PathBuf) -> io::Result<Self> {
        let mut file = open_file(path, false)?;

        let header = PitakaVaultHeader::read_from_file(&mut file)?;
        header.validate()?;

        // Collection discovery...
        let mut collections = RootCollection::read_from_file(&mut file)?;
        collections.init(&mut file)?;

        Ok(PitakaVault {
            file,
            header,
            collections,
        })
    }

    pub fn display_name(&self) -> &String {
        &self.header.display_name
    }

    pub fn accounts(&mut self) -> &mut Collection<Account> {
        self.collections.get::<Account>()
    }

    pub fn transactions(&mut self) -> &mut Collection<Transaction> {
        self.collections.get::<Transaction>()
    }

    pub fn get_account(&mut self, id: u64) -> Option<RecordSlice<Account>> {
        let res = self
            .collections
            .get::<Account>()
            .query(&mut self.file)
            .first(filter!(|account| account.id == id))
            .collect();

        match res {
            Ok(result) => result,
            Err(error) => {
                println!("{:?}", error);
                None
            }
        }
    }

    pub fn get_all_accts(&mut self) -> io::Result<Vec<RecordSlice<Account>>> {
        self.collections
            .get::<Account>()
            .query(&mut self.file)
            .collect()
    }

    pub fn get_acct_trans(&mut self, id: u64) -> io::Result<Vec<RecordSlice<Transaction>>> {
        self.collections
            .get::<Transaction>()
            .query(&mut self.file)
            .all(filter!(|transaction| transaction.item.account_id == id))
            .reverse()
            .collect()
    }

    pub fn save(&mut self) -> io::Result<()> {
        self.collections.write_to_file(&mut self.file)
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
