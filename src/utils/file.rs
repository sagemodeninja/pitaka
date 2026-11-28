use std::fs::File;
use std::io::Result;

pub trait FileHandle {
    fn read_from_file(file: &mut File) -> Result<Self>
    where
        Self: Sized;

    fn write_to_file(&mut self, file: &mut File) -> Result<()>;
}
