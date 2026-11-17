use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMoneyError {
    Empty,
    InvalidFormat,
    Overflow,
}

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "Amount is empty",
            Self::InvalidFormat => "Invalid amount format",
            Self::Overflow => "Amount is out of range",
        })
    }
}

impl Error for ParseMoneyError {}
