mod error;
pub use error::*;

use regex::Regex;
use std::fmt;
use std::{fmt::Display, str::FromStr, sync::LazyLock};

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

impl Money {
    pub const fn from_minor_units(minor_units: i64) -> Self {
        Self { minor_units }
    }

    pub const fn minor_units(self) -> i64 {
        self.minor_units
    }
}

impl FromStr for Money {
    type Err = ParseMoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(ParseMoneyError::Empty);
        }

        if let Some(groups) = AMOUNT.captures(&s) {
            let s_sign = &groups["sign"];
            let s_whole = &groups["whole"].replace(',', "");
            let s_fraction = groups
                .name("fraction")
                .map(|group| group.as_str())
                .unwrap_or("0");

            let sign = if s_sign == "-" { -1 } else { 1 };
            let whole = str_to_i64(s_whole)?;

            let fraction = str_to_i64(s_fraction)?;
            let fraction = if s_fraction.len() == 1 {
                fraction * 10
            } else {
                fraction
            };

            let minor_units = whole
                .checked_mul(100)
                .and_then(|value| value.checked_add(fraction))
                .and_then(|value| value.checked_mul(sign))
                .ok_or(ParseMoneyError::Overflow)?;

            Ok(Money::from_minor_units(minor_units))
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

fn str_to_i64(s: &str) -> Result<i64, ParseMoneyError> {
    s.parse::<i64>().map_err(|_| ParseMoneyError::Overflow)
}
