use std::borrow::Cow;
use std::marker::PhantomData;

use sqlx_core::{Execute, Executor, FromRow, TypeEncode};

use crate::{query, Arguments, Database, DefaultRuntime, Query, Runtime};

pub struct QueryAs<'q, 'a, O, Db: Database, Rt: Runtime = DefaultRuntime> {
    pub(crate) inner: Query<'q, 'a, Db, Rt>,
    output: PhantomData<O>,
}

impl<'q, 'a, Db: Database, Rt: Runtime, O: Send + Sync> Execute<'q, 'a, Db>
    for QueryAs<'q, 'a, O, Db, Rt>
{
    fn sql(&self) -> &str {
        self.inner.sql()
    }

    fn arguments(&self) -> Option<&Arguments<'a, Db>> {
        self.inner.arguments()
    }
}

impl<'q, 'a, Db: Database, Rt: Runtime, O> QueryAs<'q, 'a, O, Db, Rt> {
    pub fn bind<T: 'a + TypeEncode<Db>>(&mut self, value: &'a T) -> &mut Self {
        self.inner.bind(value);
        self
    }
}

#[cfg(feature = "async")]
impl<'q, 'a, O, Db, Rt> QueryAs<'q, 'a, O, Db, Rt>
where
    Db: Database,
    Rt: crate::Async,
    O: Send + Sync + FromRow<Db::Row>,
{
    pub async fn fetch_optional<X>(&self, mut executor: X) -> crate::Result<Option<O>>
    where
        X: Send + Executor<Rt, Database = Db>,
    {
        executor.fetch_optional(&self.inner).await?.as_ref().map(O::from_row).transpose()
    }

    pub async fn fetch_one<X>(&self, mut executor: X) -> crate::Result<O>
    where
        X: Send + Executor<Rt, Database = Db>,
    {
        O::from_row(&executor.fetch_one(&self.inner).await?)
    }

    pub async fn fetch_all<X>(&self, mut executor: X) -> crate::Result<Vec<O>>
    where
        X: Send + Executor<Rt, Database = Db>,
    {
        executor.fetch_all(self).await?.iter().map(O::from_row).collect()
    }
}

pub fn query_as<'q, 'a, O, Db: Database, Rt: Runtime>(
    sql: impl Into<Cow<'q, str>>,
) -> QueryAs<'q, 'a, O, Db, Rt> {
    QueryAs::<'q, 'a, O, Db, Rt> { inner: query(sql), output: PhantomData }
}
