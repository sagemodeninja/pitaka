use std::fs::File;
use std::io::Result;

use crate::core::document::collection::RecordPayload;
use crate::core::document::file::FileHandle;
use crate::core::types::Money;
use crate::utils::extensions::file::FileReadExtensions;
use crate::utils::extensions::file::FileWriteExtensions;
use crate::utils::extensions::string::StringExtensions;

const ID_SIZE: u32 = 8;
const ACCT_ID_SIZE: u32 = 2;
const DESC_LENGTH_SIZE: u32 = 4;
const AMOUNT_SIZE: u32 = 8;

#[derive(Clone)]
pub struct Transaction {
    pub id: u64,
    pub account_id: u16,
    pub description: String,
    pub amount: Money,
}

impl RecordPayload for Transaction {
    fn record_type() -> u8 {
        2
    }

    fn size(&self) -> u32 {
        let desc_len = self.description.len_u32();
        ID_SIZE + ACCT_ID_SIZE + DESC_LENGTH_SIZE + desc_len + AMOUNT_SIZE
    }
}

impl FileHandle for Transaction {
    fn write_to_file(&mut self, file: &mut File) -> Result<()> {
        file.write_le_u64(self.id)?;
        file.write_le_u16(self.account_id)?;
        file.write_len_pre_str(&self.description)?;
        file.write_money(&self.amount)
    }

    fn read_from_file(file: &mut File) -> Result<Self> {
        Ok(Transaction {
            id: file.read_le_u64()?,
            account_id: file.read_le_u16()?,
            description: file.read_len_pre_str()?,
            amount: file.read_money()?,
        })
    }
}
