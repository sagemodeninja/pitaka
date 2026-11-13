use std::fs::{File, OpenOptions};
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};

pub struct Transaction {
    pub description: String,
    pub amount_centavos: i64,
}

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

pub fn write_transaction(offset: u64, trans: Transaction) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).open("test.ptk")?;

    file.seek(SeekFrom::Start(offset))?;

    w_padded_str(&trans.description, &mut file)?;
    file.write_all(&trans.amount_centavos.to_le_bytes())?;

    Ok(())
}

pub fn read_transaction(offset: u64) -> io::Result<Transaction> {
    let mut file = File::open("test.ptk")?;

    file.seek(SeekFrom::Start(offset))?;

    let description = r_padded_str(&mut file)?;

    let mut amount_buf = [0u8; 8];
    file.read_exact(&mut amount_buf)?;

    let amount_centavos = i64::from_le_bytes(amount_buf);

    Ok(Transaction {
        description,
        amount_centavos,
    })
}
