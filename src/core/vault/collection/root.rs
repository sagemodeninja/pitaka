use std::fs::File;
use std::io;

use super::Transaction;
use super::{Account, Collection, CollectionHeader, CollectionType};
use crate::core::vault;
use crate::utils::{
    extensions::file::{FileCursorExtensions, FileWriteExtensions},
    file::FileHandle,
};

const COLLECTION_SIZE: u64 = 40;
const HEADER_OFFSET: u64 = vault::header::HEADER_SIZE;

pub struct RootCollection {
    headers: Collection<CollectionHeader>,
    accounts: Option<Collection<Account>>,
    transactions: Option<Collection<Transaction>>,
}

impl RootCollection {
    pub fn new(mut headers: Collection<CollectionHeader>) -> Self {
        headers.add_index("collection_type", |collection| collection.c_type.into());

        Self {
            headers,
            accounts: None,
            transactions: None,
        }
    }

    pub fn create() -> Self {
        let header = CollectionHeader::new(0, HEADER_OFFSET);
        let mut this = Self::new(Collection::new(header));

        #[rustfmt::skip]
        this
            .add::<Account>()
            .add::<Transaction>();

        this
    }

    pub fn add<T: CollectionType>(&mut self) -> &mut Self {
        let header = CollectionHeader::from_type::<T>(0);
        self.headers.add(header);
        self
    }

    pub fn accounts(&mut self, file: &mut File) -> io::Result<&mut Collection<Account>> {
        if self.accounts.is_none() {
            self.accounts = Some(self.load_collection::<Account>(file, |_| {})?);
        }

        Ok(self
            .accounts
            .as_mut()
            .expect("accounts cache is initialized"))
    }

    pub fn transactions(&mut self, file: &mut File) -> io::Result<&mut Collection<Transaction>> {
        if self.transactions.is_none() {
            self.transactions = Some(self.load_collection::<Transaction>(file, |collection| {
                collection.add_index("acct_id", |transaction| transaction.account_id.into());
            })?);
        }

        Ok(self
            .transactions
            .as_mut()
            .expect("transactions cache is initialized"))
    }

    fn load_collection<T: CollectionType>(
        &mut self,
        file: &mut File,
        configure: impl FnOnce(&mut Collection<T>),
    ) -> io::Result<Collection<T>> {
        let header = self.get_header::<T>(file)?;
        let mut collection = Collection::new(header);

        configure(&mut collection);

        collection.build_indexes(file)?;
        Ok(collection)
    }

    fn get_header<T: CollectionType>(&mut self, file: &mut File) -> io::Result<CollectionHeader> {
        let result = self
            .headers
            .query(file)
            .filter_indexed("collection_type", T::c_type().into())
            .take(1)
            .collect()?;

        if let Some(record) = result.first() {
            return Ok(record.item);
        }

        let header = CollectionHeader::from_type::<T>(0);
        self.headers.add(header);

        Ok(header)
    }
}

impl FileHandle for RootCollection {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        file.move_to(HEADER_OFFSET)?;

        let collection = Collection::read_from_file(file)?;
        let mut root = Self::new(collection);

        root.headers.build_indexes(file)?;
        Ok(root)
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        file.move_to(HEADER_OFFSET)?;

        // Reserve the header...
        file.write_zeros(COLLECTION_SIZE as usize)?;

        self.headers.write_to_file(file)
    }
}
