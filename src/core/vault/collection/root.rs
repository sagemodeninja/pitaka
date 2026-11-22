use std::fs::File;
use std::io;

use super::{
    Account, Collection, CollectionHeader, CollectionType, ManagedCollection, Transaction,
};
use crate::utils::{
    extensions::file::{FileCursorExtensions, FileWriteExtensions},
    file::FileHandle,
};

const HEADER_SIZE: u64 = 80;
const COLLECTION_SIZE: u64 = 40;

pub struct RootCollection {
    collection: Collection<ManagedCollection>,
    accounts: Option<Collection<Account>>,
    transactions: Option<Collection<Transaction>>,
}

impl RootCollection {
    pub fn new(mut collection: Collection<ManagedCollection>) -> Self {
        collection.add_index("collection_type", |collection| {
            collection.header().c_type.into()
        });

        Self {
            collection,
            accounts: None,
            transactions: None,
        }
    }

    pub fn empty() -> Self {
        let header = CollectionHeader::new(0, HEADER_SIZE);
        Self::new(Collection::new(header))
    }

    pub fn add<T: CollectionType + 'static>(&mut self) -> &mut Self {
        let header = CollectionHeader::new(T::c_type(), 0);

        let collection = Collection::<T>::new(header);
        self.collection.add(ManagedCollection::new(collection));

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

    fn load_collection<T: CollectionType + 'static>(
        &mut self,
        file: &mut File,
        configure: impl FnOnce(&mut Collection<T>),
    ) -> io::Result<Collection<T>> {
        let header = self.get_c_header::<T>(file)?;
        let mut collection = Collection::new(header);

        configure(&mut collection);

        collection.build_indexes(file)?;
        Ok(collection)
    }

    fn get_c_header<T: CollectionType + 'static>(
        &mut self,
        file: &mut File,
    ) -> io::Result<CollectionHeader> {
        let result = self
            .collection
            .query(file)
            .filter_indexed("collection_type", T::c_type().into())
            .take(1)
            .collect()?;

        if let Some(record) = result.first() {
            return Ok(*record.item.header());
        }

        let header = CollectionHeader::new(T::c_type(), 0);
        let collection = ManagedCollection::new(Collection::<T>::new(header));
        let header = *collection.header();
        self.collection.add(collection);

        Ok(header)
    }
}

impl FileHandle for RootCollection {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        file.move_to(HEADER_SIZE)?;

        let collection = Collection::read_from_file(file)?;
        let mut root = Self::new(collection);

        root.collection.build_indexes(file)?;
        Ok(root)
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        file.move_to(HEADER_SIZE)?;

        // Reserve the header...
        file.write_zeros(COLLECTION_SIZE as usize)?;

        self.collection.write_to_file(file)
    }
}
