use std::{collections::HashMap, fs::File, io};

use crate::{
    core::document::{
        collection::{RecordPayload, record::Record},
        file::FileHandle,
    },
    utils::extensions::file::FileCursorExtensions,
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

pub struct CollectionIndexBuilder<T: RecordPayload> {
    evaluators: HashMap<String, Box<dyn Fn(&T) -> IndexValue>>,
    pub indexes: HashMap<String, Index>,
}

impl<T: RecordPayload> CollectionIndexBuilder<T> {
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

    pub fn build(&mut self, count: u64, start: u64, file: &mut File) -> io::Result<()> {
        let mut o_r_next = start;

        for _ in 0..count {
            file.move_to(o_r_next)?;
            let record = Record::<T>::read_from_file(file)?;

            for (key, eval) in &self.evaluators {
                let value = eval(&record.item);
                self.indexes
                    .entry(key.clone())
                    .or_default()
                    .entry(value)
                    .or_default()
                    .push(o_r_next);
            }

            o_r_next = record.o_next;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Index> {
        self.indexes.get(key)
    }
}
