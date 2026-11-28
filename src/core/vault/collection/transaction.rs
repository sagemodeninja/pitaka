use std::fs::File;
use std::io::Result;

use chrono::DateTime;
use chrono::Utc;

use crate::core::types::Money;
use crate::core::vault::collection::CollectionType;
use crate::core::vault::collection::RecordPayload;
use crate::utils::extensions::file::FileReadExtensions;
use crate::utils::extensions::file::FileWriteExtensions;
use crate::utils::extensions::string::StringExtensions;
use crate::utils::file::FileHandle;

const ACCT_ID_SIZE: u32 = 8;
const OCCURED_AT_SIZE: u32 = 8;
const AMOUNT_SIZE: u32 = 8;

#[derive(Clone)]
pub struct Transaction {
    pub account_id: u64,
    pub occured_at: DateTime<Utc>,
    pub description: String,
    pub amount: Money,
}

impl CollectionType for Transaction {
    fn c_type() -> u8 {
        2
    }
}

impl RecordPayload for Transaction {
    fn size(&self) -> u32 {
        let desc_len = self.description.len_u32();
        ACCT_ID_SIZE + OCCURED_AT_SIZE + AMOUNT_SIZE + desc_len
    }
}

impl FileHandle for Transaction {
    fn write_to_file(&mut self, file: &mut File) -> Result<()> {
        file.write_le_u64(self.account_id)?;
        file.write(&mut self.occured_at)?;
        file.write(&mut self.amount)?;
        file.write_len_pre_str(&self.description)
    }

    fn read_from_file(file: &mut File) -> Result<Self> {
        Ok(Transaction {
            account_id: file.read_le_u64()?,
            occured_at: file.read()?,
            amount: file.read()?,
            description: file.read_len_pre_str()?,
        })
    }
}
