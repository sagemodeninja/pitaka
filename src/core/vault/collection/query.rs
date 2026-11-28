use std::io;

use super::{RecordPayload, RecordSlice, index::IndexValue};

type QueryResult<T> = Vec<RecordSlice<T>>;

pub struct QueryFilter<T: RecordPayload> {
    pub field: &'static str,
    pub value: IndexValue,
    pub evaluator: Box<dyn Fn(&RecordSlice<T>) -> bool>,
}

pub struct CollectionQueryPlan<T: RecordPayload> {
    pub skip: usize,
    pub take: usize,
    pub is_reversed: bool,
    pub filters: Vec<QueryFilter<T>>,
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
            take: usize::MAX,
            is_reversed: false,
            filters: Vec::new(),
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

    pub fn all(mut self, filter: QueryFilter<T>) -> Self {
        self.plan.filters.push(filter);
        self
    }

    pub fn first(mut self, filter: QueryFilter<T>) -> SingleResultQueryBuilder<'a, T> {
        self.plan.filters.push(filter);
        self.plan.skip = 0;
        self.plan.take = 1;

        SingleResultQueryBuilder {
            plan: self.plan,
            collector: self.collector,
        }
    }

    pub fn skip(mut self, n: usize) -> Self {
        self.plan.skip = n;
        self
    }

    pub fn take(mut self, n: usize) -> Self {
        self.plan.take = n;
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
    (|$record:ident| $receiver:ident.$first:ident$(.$rest:ident)* == $value:expr) => {{
        let value = $value;

        $crate::core::vault::collection::QueryFilter {
            field: concat!(stringify!($first) $(, "_", stringify!($rest))*),
            value: value.into(),
            evaluator: Box::new(
                move |$record: &$crate::core::vault::collection::RecordSlice<_>| {
                    $receiver.$first$(.$rest)* == value
                },
            ),
        }
    }};
}

pub(crate) use filter;
