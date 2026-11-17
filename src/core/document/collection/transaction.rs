use std::fs::File;
use std::io::Result;

use crate::core::document::collection::RecordHandle;
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

impl RecordHandle for Transaction {
    fn len(&self) -> u32 {
        let d_len = self.description.len_u32();
        ID_SIZE + ACCT_ID_SIZE + DESC_LENGTH_SIZE + d_len + AMOUNT_SIZE
    }

    fn write(&mut self, file: &mut File) -> Result<()> {
        file.write_le_u64(self.id)?;
        file.write_le_u16(self.account_id)?;
        file.write_len_pre_str(&self.description)?;
        file.write_money(&self.amount)
    }

    fn read(file: &mut File) -> Result<Self> {
        Ok(Transaction {
            id: file.read_le_u64()?,
            account_id: file.read_le_u16()?,
            description: file.read_len_pre_str()?,
            amount: file.read_money()?,
        })
    }
}
