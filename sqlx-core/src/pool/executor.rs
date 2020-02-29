use std::ops::DerefMut;

use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::StreamExt;

use crate::{
    connection::{Connect, Connection},
    describe::Describe,
    executor::Executor,
    pool::Pool,
    Cursor, Database,
};

use super::PoolConnection;
use crate::database::HasCursor;
use crate::executor::Execute;

impl<'p, C, DB> Executor<'p> for &'p Pool<C>
where
    C: Connect<Database = DB>,
    DB: Database<Connection = C>,
    DB: for<'c, 'q> HasCursor<'c, 'q, Database = DB>,
    for<'con> &'con mut C: Executor<'con>,
{
    type Database = DB;

    fn execute<'q, E>(self, query: E) -> <Self::Database as HasCursor<'p, 'q>>::Cursor
    where
        E: Execute<'q, DB>,
    {
        DB::Cursor::from_pool(self, query)
    }

    #[doc(hidden)]
    #[inline]
    fn execute_by_ref<'q, 'e, E>(
        &'e mut self,
        query: E,
    ) -> <Self::Database as HasCursor<'_, 'q>>::Cursor
    where
        E: Execute<'q, DB>,
    {
        self.execute(query)
    }
}

impl<'c, C, DB> Executor<'c> for &'c mut PoolConnection<C>
where
    C: Connect<Database = DB>,
    DB: Database<Connection = C>,
    DB: for<'c2, 'q> HasCursor<'c2, 'q, Database = DB>,
    for<'con> &'con mut C: Executor<'con>,
{
    type Database = C::Database;

    fn execute<'q, E>(self, query: E) -> <Self::Database as HasCursor<'c, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        DB::Cursor::from_connection(&mut **self, query)
    }

    #[doc(hidden)]
    #[inline]
    fn execute_by_ref<'q, 'e, E>(
        &'e mut self,
        query: E,
    ) -> <Self::Database as HasCursor<'_, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        self.execute(query)
    }
}

impl<C, DB> Executor<'static> for PoolConnection<C>
where
    C: Connect<Database = DB>,
    DB: Database<Connection = C>,
    DB: for<'c, 'q> HasCursor<'c, 'q, Database = DB>,
{
    type Database = DB;

    fn execute<'q, E>(self, query: E) -> <DB as HasCursor<'static, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        DB::Cursor::from_connection(self, query)
    }

    #[doc(hidden)]
    #[inline]
    fn execute_by_ref<'q, 'e, E>(&'e mut self, query: E) -> <DB as HasCursor<'_, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        DB::Cursor::from_connection(&mut **self, query)
    }
}
