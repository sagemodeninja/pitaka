use std::fs::File;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

use crate::core::document::PitakaDocument;

#[derive(Clone)]
pub struct Transaction {
    pub description: String,
    pub amount_centavos: i64,
}

pub struct Record<T: FileHandle> {
    pub o_prev: u64,
    pub o_next: u64,
    pub item: T,
}

pub struct Collection<T: FileHandle> {
    count: u32,
    for_append: Vec<Record<T>>,

    pub position: u64,
    pub o_first_record: u64,
    pub o_last_record: u64,
}

pub trait FileHandle: Sized {
    fn len(&self) -> u32;
    fn write(&mut self, file: &mut File, cursor: &mut u64) -> Result<()>;
    fn read(file: &mut File) -> Result<Self>;
}

impl FileHandle for Transaction {
    fn len(&self) -> u32 {
        16 + self.description.len() as u32
    }

    fn write(&mut self, file: &mut File, _: &mut u64) -> Result<()> {
        w_padded_str(&self.description, file)?;
        file.write_all(&self.amount_centavos.to_le_bytes())
    }

    fn read(file: &mut File) -> Result<Self> {
        let description = r_padded_str(file)?;

        let mut amount_buf = [0u8; 8];
        file.read_exact(&mut amount_buf)?;

        let amount_centavos = i64::from_le_bytes(amount_buf);

        Ok(Transaction {
            description,
            amount_centavos,
        })
    }
}

impl<T: FileHandle> Record<T> {
    fn new(o_prev: u64, item: T) -> Self {
        Self {
            o_prev,
            o_next: 0,
            item,
        }
    }
}

impl<T: FileHandle> FileHandle for Record<T> {
    fn len(&self) -> u32 {
        16 + self.item.len()
    }

    fn write(&mut self, file: &mut File, _doc: &mut u64) -> Result<()> {
        file.write_all(&self.o_prev.to_le_bytes())?;
        file.write_all(&self.o_next.to_le_bytes())?;
        self.item.write(file, _doc)
    }

    fn read(file: &mut File) -> Result<Self> {
        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        let o_prev = u64::from_le_bytes(buf);

        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        let o_next = u64::from_le_bytes(buf);

        let item = T::read(file)?;

        Ok(Record {
            o_prev,
            o_next,
            item,
        })
    }
}

impl<T: FileHandle> Collection<T> {
    pub fn new(pos: u64) -> Self {
        Self {
            count: 0,
            for_append: Vec::new(),

            position: pos,
            o_first_record: 0,
            o_last_record: 0,
        }
    }

    pub fn get_all(&self, file: &mut File) -> Result<Vec<T>>
    where
        T: Clone,
    {
        let mut records: Vec<T> = vec![];

        let mut cursor = self.o_first_record;
        for _ in 0..self.count {
            file.seek(SeekFrom::Start(cursor))?;

            let record = Record::<T>::read(file)?;
            records.push(record.item);

            cursor = record.o_next;
        }

        for trans in &self.for_append {
            records.push(trans.item.clone());
        }

        Ok(records)
    }

    pub fn add(&mut self, item: T) {
        let record = Record::new(self.o_last_record, item);
        self.for_append.push(record);
    }
}

impl<T: FileHandle> FileHandle for Collection<T> {
    fn len(&self) -> u32 {
        0
    }

    fn write(&mut self, file: &mut File, cursor: &mut u64) -> Result<()> {
        let append_len = self.for_append.len() as u32;
        let record_cnt = self.count + append_len;

        // Right records...
        if append_len > 0 {
            let mut records = self.for_append.iter_mut().peekable();

            let mut o_prev = self.o_last_record;
            let mut o_next = if o_prev == 0 {
                // TODO: Revisit because this might not be correct.
                self.position + 20
            } else {
                *cursor
            };

            if o_prev == 0 {
                self.o_first_record = o_next;
            } else {
                let offset = self.o_last_record + 8;
                file.seek(SeekFrom::Start(offset))?;
                file.write_all(&o_next.to_le_bytes())?;
            }

            file.seek(SeekFrom::Start(o_next))?;

            while let Some(record) = &mut records.next() {
                record.o_prev = o_prev;

                match records.peek() {
                    Some(next) => {
                        o_prev = o_next;
                        o_next += next.len() as u64;
                        record.o_next = o_next;
                    }
                    None => {
                        self.o_last_record = o_next;
                    }
                }

                record.write(file, cursor)?;
            }
        }

        // Update cursor so next append knows
        // where to start.
        *cursor = file.stream_position()? + 1;

        // Update/write collection header...
        file.seek(SeekFrom::Start(self.position))?;
        file.write_all(&record_cnt.to_le_bytes())?;
        file.write_all(&self.o_first_record.to_le_bytes())?;
        file.write_all(&self.o_last_record.to_le_bytes())?;

        Ok(())
    }

    fn read(file: &mut File) -> Result<Self> {
        let position = file.stream_position()?;

        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        let count = u32::from_le_bytes(buf);

        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        let o_first_record = u64::from_le_bytes(buf);

        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        let o_last_record = u64::from_le_bytes(buf);

        Ok(Collection {
            count,
            for_append: Vec::new(),

            position,
            o_first_record,
            o_last_record,
        })
    }
}

fn w_padded_str(value: &String, file: &mut File) -> Result<()> {
    let bytes: &[u8] = value.as_bytes();

    file.write_all(&bytes.len().to_le_bytes())?;
    file.write_all(&bytes)?;

    Ok(())
}

fn r_padded_str(file: &mut File) -> Result<String> {
    let mut len_buf = [0u8; 8];
    file.read_exact(&mut len_buf)?;

    let len = usize::from_le_bytes(len_buf);
    let mut str_buf = vec![0u8; len];
    file.read_exact(&mut str_buf)?;

    let result = String::from_utf8(str_buf);

    match result {
        Ok(text) => Ok(text),
        Err(err) => Err(Error::new(ErrorKind::InvalidData, err)),
    }
}
