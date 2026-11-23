use std::{collections::HashMap, io};

use super::{RecordPayload, RecordSlice, index::IndexValue};

type QueryResult<T> = Vec<RecordSlice<T>>;

pub struct CollectionQueryPlan {
    pub skip: u32,
    pub take: Option<u32>,
    pub is_reversed: bool,
    pub indexed_filters: HashMap<String, IndexValue>,
}

pub struct CollectionQueryBuilder<'a, T: RecordPayload> {
    plan: CollectionQueryPlan,
    collector: Box<dyn FnOnce(CollectionQueryPlan) -> io::Result<QueryResult<T>> + 'a>,
}

impl CollectionQueryPlan {
    pub fn new() -> Self {
        Self {
            skip: 0,
            take: None,
            is_reversed: false,
            indexed_filters: HashMap::new(),
        }
    }
}

impl<'a, T: RecordPayload> CollectionQueryBuilder<'a, T> {
    pub fn new(
        collector: impl FnOnce(CollectionQueryPlan) -> io::Result<QueryResult<T>> + 'a,
    ) -> Self {
        Self {
            plan: CollectionQueryPlan::new(),
            collector: Box::new(collector),
        }
    }

    pub fn filter_indexed(mut self, index: &str, value: IndexValue) -> Self {
        self.plan.indexed_filters.insert(index.into(), value);
        self
    }

    #[allow(dead_code)]
    pub fn skip(mut self, n: u32) -> Self {
        self.plan.skip = n;
        self
    }

    #[allow(dead_code)]
    pub fn take(mut self, n: u32) -> Self {
        self.plan.take = Some(n);
        self
    }

    pub fn reverse(mut self) -> Self {
        self.plan.is_reversed = true;
        self
    }

    pub fn collect(self) -> io::Result<QueryResult<T>> {
        (self.collector)(self.plan)
    }
}
