#[cfg(feature = "async")]
use futures_util::{future::BoxFuture, FutureExt};
use sqlx_core::{Execute, Executor, Result, Runtime};

use crate::{MySql, MySqlConnection, MySqlQueryResult, MySqlRow};

#[macro_use]
mod columns;

#[macro_use]
mod raw_prepare;

#[macro_use]
mod raw_query;

mod execute;
mod fetch_all;
mod fetch_optional;

impl<Rt: Runtime> Executor<Rt> for MySqlConnection<Rt> {
    type Database = MySql;

    #[cfg(feature = "async")]
    #[inline]
    fn execute<'x, 'e, 'q, 'a, E>(&'e mut self, query: E) -> BoxFuture<'x, Result<MySqlQueryResult>>
    where
        Rt: sqlx_core::Async,
        E: 'x + Execute<'q, 'a, MySql>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.execute_async(query).boxed()
    }

    #[cfg(feature = "async")]
    #[inline]
    fn fetch_all<'x, 'e, 'q, 'a, E>(&'e mut self, query: E) -> BoxFuture<'x, Result<Vec<MySqlRow>>>
    where
        Rt: sqlx_core::Async,
        E: 'x + Execute<'q, 'a, MySql>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.fetch_all_async(query).boxed()
    }

    #[cfg(feature = "async")]
    #[inline]
    fn fetch_optional<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, Result<Option<MySqlRow>>>
    where
        Rt: sqlx_core::Async,
        E: 'x + Execute<'q, 'a, MySql>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.fetch_optional_async(query).boxed()
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> sqlx_core::blocking::Executor<Rt> for MySqlConnection<Rt> {
    #[inline]
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> Result<MySqlQueryResult>
    where
        'e: 'x,
        'q: 'x,
    {
        self.execute_blocking(sql)
    }

    #[inline]
    fn fetch_all<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> Result<Vec<MySqlRow>>
    where
        'e: 'x,
        'q: 'x,
    {
        self.fetch_all_blocking(sql)
    }

    #[inline]
    fn fetch_optional<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> Result<Option<MySqlRow>>
    where
        'e: 'x,
        'q: 'x,
    {
        self.fetch_optional_blocking(sql)
    }
}
