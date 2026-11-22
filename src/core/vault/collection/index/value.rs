use std::fmt::Display;

#[derive(PartialEq, Eq, Hash)]
pub enum IndexValue {
    U8(u8),
    U64(u64),
}

impl Into<IndexValue> for u8 {
    fn into(self) -> IndexValue {
        IndexValue::U8(self)
    }
}

impl Into<IndexValue> for u64 {
    fn into(self) -> IndexValue {
        IndexValue::U64(self)
    }
}

impl Display for IndexValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            IndexValue::U8(value) => format!("{value}"),
            IndexValue::U64(value) => format!("{value}"),
        };

        write!(f, "{content}")
    }
}
