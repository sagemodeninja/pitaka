use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordDirectoryError {
    #[error("Cannot update record {0}. Record not found.")]
    Update(u64),
}

impl From<RecordDirectoryError> for io::Error {
    fn from(error: RecordDirectoryError) -> Self {
        Self::new(io::ErrorKind::InvalidData, error)
    }
}
