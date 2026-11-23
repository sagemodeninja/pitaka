use std::fs::File;
use std::io;

use indexmap::IndexMap;

use super::Transaction;
use super::{Account, AnyCollection, Collection, CollectionHeader, CollectionType};
use crate::core::vault;
use crate::utils::{extensions::file::FileCursorExtensions, file::FileHandle};

const COLLECTION_SIZE: u64 = 40;
const HEADER_OFFSET: u64 = vault::header::HEADER_SIZE;

pub struct RootCollection {
    headers: Collection<CollectionHeader>,
    collections: IndexMap<u8, AnyCollection>,
}

impl RootCollection {
    pub fn new(mut headers: Collection<CollectionHeader>) -> Self {
        headers.add_index("collection_type", |collection| collection.c_type.into());

        Self {
            headers,
            collections: IndexMap::new(),
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

    pub fn init(&mut self, file: &mut File) -> io::Result<()> {
        self.headers.build_indexes(file)?;

        self.load_collection::<Account>(file, |_| {})?;

        self.load_collection::<Transaction>(file, |collection| {
            collection.add_index("acct_id", |transaction| transaction.account_id.into());
        })?;

        Ok(())
    }

    pub fn add<T: CollectionType>(&mut self) -> &mut Self {
        let header = CollectionHeader::from_type::<T>(0);
        self.headers.add(header);
        self
    }

    pub fn get<T: CollectionType + 'static>(&mut self) -> &mut Collection<T> {
        self.collections
            .get_mut(&T::c_type())
            .and_then(|collection| collection.downcast_mut::<T>())
            .expect("cache is initialized")
    }

    fn load_collection<T: CollectionType + 'static>(
        &mut self,
        file: &mut File,
        configure: impl FnOnce(&mut Collection<T>),
    ) -> io::Result<()> {
        let header = self.get_header::<T>(file)?;
        let mut collection = Collection::new(header);

        configure(&mut collection);
        collection.build_indexes(file)?;

        self.collections
            .insert(T::c_type(), AnyCollection::new(collection));
        Ok(())
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
        Ok(Self::new(collection))
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let root_header_end = HEADER_OFFSET + COLLECTION_SIZE;

        if file.metadata()?.len() < root_header_end {
            file.set_len(root_header_end)?;
        }

        self.headers.write_to_file(file)?;

        for (&c_type, collection) in &mut self.collections {
            let header = collection.header_mut();

            // Current naive approach to syncing positions.
            if header.position == 0 {
                header.position = self
                    .headers
                    .query(file)
                    .collect()?
                    .into_iter()
                    .find(|record| record.item.c_type == c_type)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "missing collection header")
                    })?
                    .item
                    .position;
            }

            collection.write_to_file(file)?;
        }

        Ok(())
    }
}
