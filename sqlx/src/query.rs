use std::borrow::Cow;
use std::marker::PhantomData;

use sqlx_core::{Execute, Executor, TypeEncode};

use crate::{Arguments, Database, DefaultRuntime, Runtime};

pub struct Query<'q, 'a, Db: Database, Rt: Runtime = DefaultRuntime> {
    sql: Cow<'q, str>,
    arguments: Arguments<'a, Db>,
    runtime: PhantomData<Rt>,
}

impl<'q, 'a, Db: Database, Rt: Runtime> Execute<'q, 'a, Db> for Query<'q, 'a, Db, Rt> {
    fn sql(&self) -> &str {
        &self.sql
    }

    fn arguments(&self) -> Option<&Arguments<'a, Db>> {
        Some(&self.arguments)
    }
}

impl<'q, 'a, Db: Database, Rt: Runtime> Query<'q, 'a, Db, Rt> {
    pub fn bind<T: 'a + TypeEncode<Db>>(&mut self, value: &'a T) -> &mut Self {
        self.arguments.add(value);
        self
    }

    pub fn bind_unchecked<T: 'a + TypeEncode<Db>>(&mut self, value: &'a T) -> &mut Self {
        self.arguments.add_unchecked(value);
        self
    }

    pub fn bind_named<T: 'a + TypeEncode<Db>>(&mut self, name: &'a str, value: &'a T) -> &mut Self{ //we don't use AsRef<str> since that breaks lifetimes
        self.arguments.add_as(name, value);
        self
    }
}

#[cfg(feature = "async")]
impl<'q, 'a, Db: Database, Rt: crate::Async> Query<'q, 'a, Db, Rt> {
    pub async fn execute<X>(&self, mut executor: X) -> crate::Result<Db::QueryResult>
    where
        X: Executor<Rt, Database = Db>,
    {
        executor.execute(self).await
    }

    pub async fn fetch_optional<X>(&self, mut executor: X) -> crate::Result<Option<Db::Row>>
    where
        X: Executor<Rt, Database = Db>,
    {
        executor.fetch_optional(self).await
    }

    pub async fn fetch_one<X>(&self, mut executor: X) -> crate::Result<Db::Row>
    where
        X: Executor<Rt, Database = Db>,
    {
        executor.fetch_one(self).await
    }

    pub async fn fetch_all<X>(&self, mut executor: X) -> crate::Result<Vec<Db::Row>>
    where
        X: Executor<Rt, Database = Db>,
    {
        executor.fetch_all(self).await
    }
}

pub fn query<'q, 'a, Db: Database, Rt: Runtime>(
    sql: impl Into<Cow<'q, str>>,
) -> Query<'q, 'a, Db, Rt> {
    Query::<'q, 'a, Db, Rt> { sql: sql.into(), arguments: Arguments::new(), runtime: PhantomData }
}
