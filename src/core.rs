use std::fs::{File, OpenOptions};
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};

fn w_padded_str(value: &String, file: &mut File) -> io::Result<()> {
    let bytes: &[u8] = value.as_bytes();

    file.write_all(&bytes.len().to_le_bytes())?;
    file.write_all(&bytes)?;

    Ok(())
}

fn r_padded_str(file: &mut File) -> io::Result<String> {
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

pub trait FileHandle: Sized {
    fn len(&self) -> u32;
    fn write(&mut self, file: &mut File) -> io::Result<()>;
    fn read(offset: u64, file: &mut File) -> io::Result<Self>;
}

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
    len: u32,
    for_append: Vec<Record<T>>,
    last_record: Option<Record<T>>,

    pub position: u64,
    pub o_first_record: u64,
    pub o_last_record: u64,
}

impl FileHandle for Transaction {
    fn len(&self) -> u32 {
        8 + self.description.len() as u32
    }

    fn write(&mut self, file: &mut File) -> io::Result<()> {
        w_padded_str(&self.description, file)?;
        file.write_all(&self.amount_centavos.to_le_bytes())
    }

    fn read(offset: u64, file: &mut File) -> io::Result<Self> {
        file.seek(SeekFrom::Start(offset))?;

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

    fn write(&mut self, file: &mut File) -> io::Result<()> {
        file.write_all(&self.o_prev.to_le_bytes())?;
        file.write_all(&self.o_next.to_le_bytes())?;
        self.item.write(file)
    }

    fn read(offset: u64, file: &mut File) -> io::Result<Self> {
        todo!()
    }
}

impl<T: FileHandle> Collection<T> {
    pub fn new(pos: u64) -> Self {
        Self {
            len: 0,
            for_append: Vec::new(),
            last_record: None,

            position: pos,
            o_first_record: 0,
            o_last_record: 0,
        }
    }

    pub fn add(&mut self, item: T) {
        let record = Record::new(self.o_last_record, item);
        self.for_append.push(record);
        self.len += 1;
    }
}

impl<T: FileHandle> FileHandle for Collection<T> {
    fn len(&self) -> u32 {
        0
    }

    fn write(&mut self, file: &mut File) -> io::Result<()> {
        // Right records...
        let mut records = self.for_append.iter_mut().peekable();

        let mut o_prev = self.o_last_record;
        let mut o_next = file.stream_position()?;

        // if let Some(record) = &self.last_record {
        //     o_next += record.len() as u64;
        // }

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

            record.write(file)?;
        }

        // Update/write collection header...
        file.seek(SeekFrom::Start(self.position))?;
        file.write_all(&self.len.to_le_bytes())?;
        file.write_all(&self.o_first_record.to_le_bytes())?;
        file.write_all(&self.o_last_record.to_le_bytes())?;

        Ok(())
    }

    fn read(offset: u64, file: &mut File) -> io::Result<Self> {
        todo!()
        // file.seek(SeekFrom::Start(offset))?;

        // let description = r_padded_str(file)?;

        // let mut amount_buf = [0u8; 8];
        // file.read_exact(&mut amount_buf)?;

        // let amount_centavos = i64::from_le_bytes(amount_buf);

        // Ok(Transaction {
        //     description,
        //     amount_centavos,
        // })
    }
}

pub fn write_transaction(trans: &mut Collection<Transaction>) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).open("test.ptk")?;
    trans.write(&mut file)?;
    Ok(())
}

pub fn read_transaction(offset: u64) -> io::Result<Transaction> {
    let mut file = File::open("test.ptk")?;
    Transaction::read(offset, &mut file)
}
