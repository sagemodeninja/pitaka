use regex::Regex;
use std::fs::File;
use std::{fmt, io};
use std::{fmt::Display, str::FromStr, sync::LazyLock};
use thiserror::Error;

use crate::utils::extensions::file::{FileReadExtensions, FileWriteExtensions};
use crate::utils::file::FileHandle;

static AMOUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<sign>[+-]?)(?<whole>[0-9]{1,3}(?:,[0-9]{3})+|[0-9]+)(?:\.(?<fraction>[0-9]{1,2}))?$",
    )
    .expect("Invalid amount regex")
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money {
    minor_units: i64,
}

impl FromStr for Money {
    type Err = ParseMoneyError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let string = string.trim();

        if string.is_empty() {
            return Err(ParseMoneyError::Empty);
        }

        if let Some(groups) = AMOUNT.captures(&string) {
            let sign = if &groups["sign"] == "-" { -1 } else { 1 };
            let whole = str_to_i64(&groups["whole"])?;

            let s_fraction = groups
                .name("fraction")
                .map(|group| group.as_str())
                .unwrap_or("0");
            let mut fraction = str_to_i64(s_fraction)?;

            if s_fraction.len() == 1 {
                fraction *= 10;
            }

            let minor_units = whole
                .checked_mul(100)
                .and_then(|value| value.checked_add(fraction))
                .and_then(|value| value.checked_mul(sign))
                .ok_or(ParseMoneyError::Overflow)?;

            Ok(Money { minor_units })
        } else {
            return Err(ParseMoneyError::InvalidFormat);
        }
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.minor_units < 0 { "-" } else { "" };
        let magnitude = self.minor_units.unsigned_abs();

        let pesos = magnitude / 100;
        let centavos = magnitude % 100;

        write!(f, "{sign}₱{pesos}.{centavos:02}")
    }
}

impl FileHandle for Money {
    fn read_from_file(file: &mut File) -> io::Result<Self> {
        let minor_units = file.read_le_i64()?;
        Ok(Money { minor_units })
    }

    fn write_to_file(&mut self, file: &mut File) -> io::Result<()> {
        let minor_units = self.minor_units;
        file.write_le_i64(minor_units)
    }
}

fn str_to_i64(string: &str) -> Result<i64, ParseMoneyError> {
    string
        .replace(',', "")
        .parse::<i64>()
        .map_err(|_| ParseMoneyError::Overflow)
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMoneyError {
    #[error("Amount is empty.")]
    Empty,

    #[error("Invalid amount format.")]
    InvalidFormat,

    #[error("Amount is out of range.")]
    Overflow,
}
