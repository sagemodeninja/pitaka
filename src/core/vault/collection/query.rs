use std::{collections::HashMap, io};

use super::{RecordPayload, RecordSlice, index::IndexValue};

type QueryResult<T> = Vec<RecordSlice<T>>;

pub struct QueryFilter<T: RecordPayload> {
    pub field: &'static str,
    pub value: IndexValue,
    pub evaluator: Box<dyn Fn(&RecordSlice<T>) -> bool>,
}

pub struct CollectionQueryPlan<T: RecordPayload> {
    pub skip: u32,
    pub take: Option<u32>,
    pub is_reversed: bool,
    pub filters: Vec<QueryFilter<T>>,
    pub indexed_filters: HashMap<String, IndexValue>,
}

pub struct CollectionQueryBuilder<'a, T: RecordPayload> {
    plan: CollectionQueryPlan<T>,
    collector: Box<dyn FnOnce(CollectionQueryPlan<T>) -> io::Result<QueryResult<T>> + 'a>,
}

pub struct SingleResultQueryBuilder<'a, T: RecordPayload> {
    plan: CollectionQueryPlan<T>,
    collector: Box<dyn FnOnce(CollectionQueryPlan<T>) -> io::Result<QueryResult<T>> + 'a>,
}

impl<T: RecordPayload> CollectionQueryPlan<T> {
    pub fn new() -> Self {
        Self {
            skip: 0,
            take: None,
            is_reversed: false,
            filters: Vec::new(),
            indexed_filters: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
impl<'a, T: RecordPayload> SingleResultQueryBuilder<'a, T> {
    pub fn reverse(mut self) -> Self {
        self.plan.is_reversed = true;
        self
    }

    pub fn collect(self) -> io::Result<Option<RecordSlice<T>>> {
        let results = (self.collector)(self.plan)?;
        Ok(results.into_iter().next())
    }
}

#[allow(dead_code)]
impl<'a, T: RecordPayload> CollectionQueryBuilder<'a, T> {
    pub fn new(
        collector: impl FnOnce(CollectionQueryPlan<T>) -> io::Result<QueryResult<T>> + 'a,
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

    pub fn first(mut self, filter: QueryFilter<T>) -> SingleResultQueryBuilder<'a, T> {
        self.plan.filters.push(filter);
        self.plan.skip = 0;
        self.plan.take = Some(1);

        SingleResultQueryBuilder {
            plan: self.plan,
            collector: self.collector,
        }
    }

    pub fn skip(mut self, n: u32) -> Self {
        self.plan.skip = n;
        self
    }

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

macro_rules! filter {
    (|$record:ident| $receiver:ident.$field:ident == $value:expr) => {{
        let value = $value;
        $crate::core::vault::collection::QueryFilter {
            field: stringify!($field),
            value: value.into(),
            evaluator: Box::new(
                move |$record: &$crate::core::vault::collection::RecordSlice<_>| {
                    $receiver.$field == value
                },
            ),
        }
    }};
}

pub(crate) use filter;
