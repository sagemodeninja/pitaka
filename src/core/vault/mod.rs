pub mod collection;
mod header;

use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;

use crate::utils::file::FileHandle;
use collection::{Account, Collection, Record, RootCollection, Transaction};
use header::PitakaVaultHeader;

pub struct PitakaVault {
    file: File,
    header: PitakaVaultHeader,
    collections: RootCollection,
}

impl PitakaVault {
    // SEE: /docs/develop/file_layout.md
    pub fn create(name: impl Into<String>, path: &PathBuf) -> io::Result<Self> {
        let mut file = open_file(path, true)?;

        let mut header = PitakaVaultHeader::new(name);
        let mut collections = RootCollection::empty();

        #[rustfmt::skip]
        collections
            .add::<Account>()
            .add::<Transaction>();

        // Write to file...
        header.write_to_file(&mut file)?;
        collections.write_to_file(&mut file)?;

        Ok(Self {
            file,
            header,
            collections,
        })
    }

    pub fn open(path: &PathBuf) -> io::Result<Self> {
        let file = open_file(path, false)?;
        read_vault(file)
    }

    pub fn display_name(&self) -> &String {
        &self.header.display_name
    }

    pub fn accounts(&mut self) -> io::Result<&mut Collection<Account>> {
        self.collections.accounts(&mut self.file)
    }

    pub fn transactions(&mut self) -> io::Result<&mut Collection<Transaction>> {
        self.collections.transactions(&mut self.file)
    }

    pub fn get_all_accts(&mut self) -> io::Result<Vec<Record<Account>>> {
        self.collections
            .accounts(&mut self.file)?
            .query(&mut self.file)
            .collect()
    }

    pub fn get_acct_trans(&mut self, id: u64) -> io::Result<Vec<Record<Transaction>>> {
        self.collections
            .transactions(&mut self.file)?
            .query(&mut self.file)
            .filter_indexed("acct_id", id.into())
            .reverse()
            .collect()
    }

    pub fn save(&mut self) -> io::Result<()> {
        self.collections
            .accounts(&mut self.file)?
            .write_to_file(&mut self.file)?;

        self.collections
            .transactions(&mut self.file)?
            .write_to_file(&mut self.file)?;

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
fn read_vault(mut file: File) -> io::Result<PitakaVault> {
    let header = PitakaVaultHeader::read_from_file(&mut file)?;

    header.validate()?;

    // Collection discovery...
    let collections = RootCollection::read_from_file(&mut file)?;

    // TODO: Reimplement...
    // transactions.add_index("acct_id", |trans| trans.account_id.into());
    // accounts.build_indexes(&mut file)?;
    // transactions.build_indexes(&mut file)?;

    Ok(PitakaVault {
        file,
        header,
        collections,
    })
}
