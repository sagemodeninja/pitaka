use std::fmt::Display;

#[derive(PartialEq, Eq, Hash)]
pub enum IndexValue {
    U64(u64),
}

impl Into<IndexValue> for u64 {
    fn into(self) -> IndexValue {
        IndexValue::U64(self)
    }
}

impl Display for IndexValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            IndexValue::U64(value) => format!("{value}"),
        };

        write!(f, "{content}")
    }
}
