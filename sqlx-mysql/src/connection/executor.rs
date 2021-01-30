#[cfg(feature = "async")]
use futures_util::{future::BoxFuture, FutureExt};
use sqlx_core::{Executor, Result, Runtime};

use crate::{MySql, MySqlConnection, MySqlQueryResult, MySqlRow};

#[macro_use]
mod columns;

#[macro_use]
mod execute;

impl<Rt: Runtime> Executor<Rt> for MySqlConnection<Rt> {
    type Database = MySql;

    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<MySqlQueryResult>>
    where
        Rt: sqlx_core::Async,
        'e: 'x,
        'q: 'x,
    {
        self.execute_async(sql).boxed()
    }

    fn fetch_all<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<Vec<MySqlRow>>>
    where
        Rt: sqlx_core::Async,
        'e: 'x,
        'q: 'x,
    {
        todo!()
    }

    fn fetch_optional<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> BoxFuture<'x, Result<Option<MySqlRow>>>
    where
        Rt: sqlx_core::Async,
        'e: 'x,
        'q: 'x,
    {
        todo!()
    }

    fn fetch_one<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<MySqlRow>>
    where
        Rt: sqlx_core::Async,
        'e: 'x,
        'q: 'x,
    {
        todo!()
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> sqlx_core::blocking::Executor<Rt> for MySqlConnection<Rt> {
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> Result<MySqlQueryResult>
    where
        'e: 'x,
        'q: 'x,
    {
        self.execute_blocking(sql)
    }
}
