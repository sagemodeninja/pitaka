use crate::core::vault::collection::{Record, RecordPayload};

use super::value::IndexValue;
use std::collections::HashMap;

type Index = HashMap<IndexValue, Vec<u64>>;

pub struct CollectionIndexBuilder<T: RecordPayload> {
    evaluators: HashMap<String, Box<dyn Fn(&T) -> IndexValue>>,
    indexes: HashMap<String, Index>,
}

impl<T: RecordPayload> CollectionIndexBuilder<T> {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
            evaluators: HashMap::new(),
        }
    }

    pub fn add_index(&mut self, key: &str, eval: impl Fn(&T) -> IndexValue + 'static) -> &mut Self {
        self.evaluators.insert(key.into(), Box::new(eval));
        self
    }

    pub fn get(&self, key: &str) -> Option<&Index> {
        self.indexes.get(key)
    }

    pub fn insert(&mut self, record: &Record<T>) {
        for (key, eval) in &self.evaluators {
            self.indexes
                .entry(key.clone())
                .or_default()
                .entry(eval(&record.item))
                .or_default()
                .push(record.id);
        }
    }
}
