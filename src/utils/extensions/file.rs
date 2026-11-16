use std::fs::File;
use std::io::{self, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

pub trait FileCursorExtensions {
    fn move_to(&mut self, pos: u64) -> Result<u64>;
    fn move_for(&mut self, steps: i64) -> Result<u64>;
}

pub trait FileReadExtensions {
    fn read_4_bytes(&mut self) -> Result<[u8; 4]>;
    fn read_8_bytes(&mut self) -> Result<[u8; 8]>;

    fn read_le_u8(&mut self) -> Result<u8>;
    fn read_le_u16(&mut self) -> Result<u16>;
    fn read_le_u32(&mut self) -> Result<u32>;
    fn read_le_i64(&mut self) -> Result<i64>;
    fn read_le_u64(&mut self) -> Result<u64>;

    fn read_len_pre_str(&mut self) -> Result<String>;
}

pub trait FileWriteExtensions {
    fn write_le_u16(&mut self, value: u16) -> Result<()>;
    fn write_le_i64(&mut self, value: i64) -> Result<()>;
    fn write_le_u64(&mut self, value: u64) -> Result<()>;

    fn write_len_pre_str(&mut self, value: &String) -> Result<()>;
}

impl FileCursorExtensions for File {
    fn move_to(&mut self, pos: u64) -> Result<u64> {
        self.seek(SeekFrom::Start(pos))
    }

    fn move_for(&mut self, steps: i64) -> Result<u64> {
        self.seek(SeekFrom::Current(steps))
    }
}

impl FileReadExtensions for File {
    // Bytes...
    fn read_4_bytes(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_8_bytes(&mut self) -> Result<[u8; 8]> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    // Integers...
    fn read_le_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(u8::from_le_bytes(buf))
    }

    fn read_le_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_le_u32(&mut self) -> Result<u32> {
        let buf = self.read_4_bytes()?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_le_i64(&mut self) -> Result<i64> {
        let buf = self.read_8_bytes()?;
        Ok(i64::from_le_bytes(buf))
    }

    fn read_le_u64(&mut self) -> Result<u64> {
        let buf = self.read_8_bytes()?;
        Ok(u64::from_le_bytes(buf))
    }

    // String...
    fn read_len_pre_str(&mut self) -> Result<String> {
        let len = self.read_le_u32()? as usize;

        let mut str_buf = vec![0u8; len];
        self.read_exact(&mut str_buf)?;

        String::from_utf8(str_buf).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

impl FileWriteExtensions for File {
    fn write_le_u16(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_le_i64(&mut self, value: i64) -> Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_le_u64(&mut self, value: u64) -> Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_len_pre_str(&mut self, text: &String) -> Result<()> {
        let len = u32::try_from(text.len()).map_err(|_| {
            Error::new(
                ErrorKind::InvalidInput,
                "String exceeds u32 byte length limit",
            )
        })?;
        let bytes: &[u8] = text.as_bytes();

        self.write_all(&len.to_le_bytes())?;
        self.write_all(&bytes)?;

        Ok(())
    }
}
