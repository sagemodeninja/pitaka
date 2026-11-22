use std::{
    fs::File,
    io::{self, Seek},
    rc::Rc,
};

use crate::{
    core::vault::collection::CollectionHeader,
    utils::{
        extensions::file::{FileCursorExtensions, FileReadExtensions},
        file::FileHandle,
    },
};

use super::{Account, COLLECTION_SIZE, Collection, CollectionHandle, RecordPayload};

#[derive(Clone)]
pub struct ManagedCollection {
    collection: Rc<dyn CollectionHandle>,
}

impl ManagedCollection {
    pub fn new(collection: impl CollectionHandle + 'static) -> Self {
        Self {
            collection: Rc::new(collection),
        }
    }

    pub fn header(&self) -> &CollectionHeader {
        self.collection.header()
    }
}

impl FileHandle for ManagedCollection {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        let position = file.stream_position()?;

        let c_type = file.read_le_u8()?;
        file.move_to(position)?;

        let collection = match c_type {
            1 => <Collection<Account> as FileHandle>::read_from_file(file)?,
            2 => <Collection<Account> as FileHandle>::read_from_file(file)?,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown collection type",
                ));
            }
        };

        Ok(Self::new(collection))
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let mut header = *self.header();
        header.write_to_file(file)
    }
}

impl RecordPayload for ManagedCollection {
    fn size(&self) -> u32 {
        COLLECTION_SIZE as u32
    }
}
