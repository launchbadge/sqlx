use std::ops::DerefMut;

use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::StreamExt;

use crate::{
    connection::{Connect, Connection},
    describe::Describe,
    executor::Executor,
    pool::Pool,
    Database,
};

use super::PoolConnection;
use crate::database::HasCursor;
use crate::executor::Execute;

impl<'p, C> Executor<'p> for &'p Pool<C>
where
    C: Connect,
    for<'con> &'con mut C: Executor<'con>,
{
    type Database = C::Database;

    fn execute<'q, E>(self, query: E) -> <Self::Database as HasCursor<'p>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        todo!()
    }

    fn execute_by_ref<'q, 'e, E>(
        &'e mut self,
        query: E,
    ) -> <Self::Database as HasCursor<'_>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        todo!()
    }
}

impl<'c, C> Executor<'c> for &'c mut PoolConnection<C>
where
    C: Connect,
    for<'con> &'con mut C: Executor<'con>,
{
    type Database = C::Database;

    fn execute<'q, E>(self, query: E) -> <Self::Database as HasCursor<'c>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        todo!()
    }

    fn execute_by_ref<'q, 'e, E>(
        &'e mut self,
        query: E,
    ) -> <Self::Database as HasCursor<'_>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        todo!()
    }
}

impl<C> Executor<'static> for PoolConnection<C>
where
    C: Connect,
    // for<'con> &'con mut C: Executor<'con>,
{
    type Database = C::Database;

    fn execute<'q, E>(self, query: E) -> <Self::Database as HasCursor<'static>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        unimplemented!()
    }

    fn execute_by_ref<'q, 'e, E>(
        &'e mut self,
        query: E,
    ) -> <Self::Database as HasCursor<'_>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        todo!()
    }
}
