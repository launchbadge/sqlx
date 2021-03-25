#[cfg(feature = "async")]
use futures_util::{future::BoxFuture, FutureExt};
use sqlx_core::{Execute, Executor, Result, Runtime};

use crate::protocol::backend::ReadyForQuery;
use crate::{PgConnection, PgQueryResult, PgRow, Postgres};

#[macro_use]
mod raw_prepare;

#[macro_use]
mod raw_query;

mod execute;
mod fetch_all;
mod fetch_optional;

impl<Rt: Runtime> PgConnection<Rt> {
    pub(crate) fn handle_ready_for_query(&mut self, rq: ReadyForQuery) {
        self.transaction_status = rq.transaction_status;

        debug_assert!(self.pending_ready_for_query_count > 0);
        self.pending_ready_for_query_count -= 1;
    }
}

impl<Rt: Runtime> Executor<Rt> for PgConnection<Rt> {
    type Database = Postgres;

    #[cfg(feature = "async")]
    #[inline]
    fn execute<'x, 'e, 'q, 'v, X>(&'e mut self, query: X) -> BoxFuture<'x, Result<PgQueryResult>>
    where
        Rt: sqlx_core::Async,
        X: 'x + Execute<'q, 'v, Postgres>,
        'e: 'x,
        'q: 'x,
        'v: 'x,
    {
        Box::pin(self.execute_async(query))
    }

    #[cfg(feature = "async")]
    #[inline]
    fn fetch_all<'x, 'e, 'q, 'v, X>(&'e mut self, query: X) -> BoxFuture<'x, Result<Vec<PgRow>>>
    where
        Rt: sqlx_core::Async,
        X: 'x + Execute<'q, 'v, Postgres>,
        'e: 'x,
        'q: 'x,
        'v: 'x,
    {
        Box::pin(self.fetch_all_async(query))
    }

    #[cfg(feature = "async")]
    #[inline]
    fn fetch_optional<'x, 'e, 'q, 'v, X>(
        &'e mut self,
        query: X,
    ) -> BoxFuture<'x, Result<Option<PgRow>>>
    where
        Rt: sqlx_core::Async,
        X: 'x + Execute<'q, 'v, Postgres>,
        'e: 'x,
        'q: 'x,
        'v: 'x,
    {
        Box::pin(self.fetch_optional_async(query))
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> sqlx_core::blocking::Executor<Rt> for PgConnection<Rt> {
    #[inline]
    fn execute<'x, 'e, 'q, 'v, X>(&'e mut self, query: X) -> Result<PgQueryResult>
    where
        X: 'x + Execute<'q, 'v, Postgres>,
        'e: 'x,
        'q: 'x,
        'v: 'x,
    {
        self.execute_blocking(query)
    }

    #[inline]
    fn fetch_all<'x, 'e, 'q, 'v, X>(&'e mut self, query: X) -> Result<Vec<PgRow>>
    where
        X: 'x + Execute<'q, 'v, Postgres>,
        'e: 'x,
        'q: 'x,
        'v: 'x,
    {
        self.fetch_all_blocking(query)
    }

    #[inline]
    fn fetch_optional<'x, 'e, 'q, 'v, X>(&'e mut self, query: X) -> Result<Option<PgRow>>
    where
        X: 'x + Execute<'q, 'v, Postgres>,
        'e: 'x,
        'q: 'x,
        'v: 'x,
    {
        self.fetch_optional_blocking(query)
    }
}
