use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("Record with ID '{0}' not found.")]
    RecordNotFound(u64),

    #[error("{0}")]
    InvalidOperation(String),
}

impl From<CollectionError> for io::Error {
    fn from(error: CollectionError) -> Self {
        Self::new(io::ErrorKind::InvalidData, error)
    }
}
