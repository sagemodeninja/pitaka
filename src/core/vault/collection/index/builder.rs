use crate::core::vault::collection::RecordPayload;

use super::value::IndexValue;
use std::collections::HashMap;

type Index = HashMap<IndexValue, Vec<u64>>;

pub struct CollectionIndex<T: RecordPayload> {
    pub field: &'static str,
    pub evaluator: Box<dyn Fn(&T) -> IndexValue>,
}

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

    pub fn add_index(&mut self, index: CollectionIndex<T>) -> &mut Self {
        self.evaluators.insert(index.field.into(), index.evaluator);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Index> {
        self.indexes.get(key)
    }

    pub fn insert(&mut self, id: u64, item: &T) {
        for (key, eval) in &self.evaluators {
            let value = eval(&item);

            self.indexes
                .entry(key.clone())
                .or_default()
                .entry(value)
                .or_default()
                .push(id);
        }
    }
}

macro_rules! index {
    (|$record:ident| $receiver:ident.$first:ident$(.$rest:ident)*) => {
        $crate::core::vault::collection::CollectionIndex {
            field: concat!(stringify!($first) $(, "_", stringify!($rest))*),
            evaluator: Box::new(|$record| $receiver.$first$(.$rest)*.into())
        }
    };
}

pub(crate) use index;
