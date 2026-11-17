use std::{collections::HashMap, fs::File, io};

use crate::{
    core::document::collection::RecordHandle,
    utils::extensions::file::{FileCursorExtensions, FileReadExtensions},
};

type Index = HashMap<IndexValue, Vec<u64>>;

#[derive(PartialEq, Eq, Hash)]
pub enum IndexValue {
    U16(u16),
}

impl Into<IndexValue> for u16 {
    fn into(self) -> IndexValue {
        IndexValue::U16(self)
    }
}

pub struct CollectionIndexBuilder<T: RecordHandle> {
    evaluators: HashMap<String, Box<dyn Fn(&T) -> IndexValue>>,
    pub indexes: HashMap<String, Index>,
}

impl<T: RecordHandle> CollectionIndexBuilder<T> {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
            evaluators: HashMap::new(),
        }
    }

    pub fn index(&mut self, key: &str, eval: impl Fn(&T) -> IndexValue + 'static) -> &mut Self {
        self.evaluators.insert(key.into(), Box::new(eval));
        self
    }

    pub fn build(&mut self, head: u64, file: &mut File) -> io::Result<()> {
        file.move_to(head + 17)?;

        let count = file.read_le_u64()?;
        let mut o_r_curr = file.read_le_u64()?;

        for _ in 0..count {
            file.move_to(o_r_curr + 8)?;
            let o_r_next = file.read_le_u64()?;

            let item = T::read(file)?;
            for (key, eval) in &self.evaluators {
                let value = eval(&item);
                self.indexes
                    .entry(key.clone())
                    .or_default()
                    .entry(value)
                    .or_default()
                    .push(o_r_curr);
            }

            o_r_curr = o_r_next;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Index> {
        self.indexes.get(key)
    }
}
