use std::io;

pub struct CollectionQueryPlan {
    pub skip: u32,
    pub take: u32,
}

impl CollectionQueryPlan {
    pub fn new() -> Self {
        Self { skip: 0, take: 0 }
    }
}

pub struct CollectionQueryBuilder<'a, T> {
    plan: CollectionQueryPlan,
    collector: Box<dyn FnOnce(CollectionQueryPlan) -> io::Result<Vec<T>> + 'a>,
}

impl<'a, T> CollectionQueryBuilder<'a, T> {
    pub fn new(exec: impl FnOnce(CollectionQueryPlan) -> io::Result<Vec<T>> + 'a) -> Self {
        Self {
            collector: Box::new(exec),
            plan: CollectionQueryPlan::new(),
        }
    }

    pub fn skip(mut self, n: u32) -> Self {
        self.plan.skip = n;
        self
    }

    pub fn take(mut self, n: u32) -> Self {
        self.plan.take = n;
        self
    }

    pub fn collect(self) -> io::Result<Vec<T>> {
        (self.collector)(self.plan)
    }
}
