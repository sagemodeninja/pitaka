use std::{any::Any, fs::File, io};

use crate::utils::file::FileHandle;

use super::{Collection, CollectionHeader, RecordPayload};

// Object
pub trait CollectionObject {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn header_mut(&mut self) -> &mut CollectionHeader;
    fn write_to_file(&mut self, file: &mut File) -> io::Result<()>;
}

impl<T: RecordPayload + 'static> CollectionObject for Collection<T> {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn header_mut(&mut self) -> &mut CollectionHeader {
        &mut self.header
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        if self.header.position == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "collection header has no file position",
            ));
        }

        FileHandle::write_to_file(self, file)
    }
}

// Any
pub struct AnyCollection(Box<dyn CollectionObject>);

impl AnyCollection {
    pub fn new<T: RecordPayload + 'static>(collection: Collection<T>) -> Self {
        Self(Box::new(collection))
    }

    pub fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        self.0.write_to_file(file)
    }

    pub(super) fn header_mut(&mut self) -> &mut CollectionHeader {
        self.0.header_mut()
    }

    pub fn downcast_mut<T: RecordPayload + 'static>(&mut self) -> Option<&mut Collection<T>> {
        self.0.as_any_mut().downcast_mut::<Collection<T>>()
    }
}
