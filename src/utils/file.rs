use std::fs::File;
use std::io::Result;

pub trait FileHandle: Sized {
    fn read_from_file(file: &mut File) -> Result<Self>;
    fn write_to_file(&mut self, file: &mut File) -> Result<()>;
}
