use crate::{query::IntoQueryParameters, Backend, Executor, FromSqlRow};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::marker::PhantomData;

pub struct CompiledSql<P, O, DB> {
    #[doc(hidden)]
    pub query: &'static str,
    #[doc(hidden)]
    pub params: P,
    #[doc(hidden)]
    pub output: PhantomData<O>,
    pub backend: PhantomData<DB>,
}

impl<DB, P, O> CompiledSql<P, O, DB>
where
    DB: Backend,
    P: IntoQueryParameters<DB> + Send,
    O: FromSqlRow<DB> + Send + Unpin,
{
    #[inline]
    pub fn execute<'e, E: 'e>(self, executor: &'e mut E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Executor<Backend = DB>,
        DB: 'e,
        P: 'e,
        O: 'e,
    {
        executor.execute(self.query, self.params)
    }

    #[inline]
    pub fn fetch<'e, E: 'e>(self, executor: &'e mut E) -> BoxStream<'e, crate::Result<O>>
    where
        E: Executor<Backend = DB>,
        DB: 'e,
        P: 'e,
        O: 'e,
    {
        executor.fetch(self.query, self.params)
    }

    #[inline]
    pub fn fetch_all<'e, E: 'e>(self, executor: &'e mut E) -> BoxFuture<'e, crate::Result<Vec<O>>>
    where
        E: Executor<Backend = DB>,
        DB: 'e,
        P: 'e,
        O: 'e,
    {
        executor.fetch_all(self.query, self.params)
    }

    #[inline]
    pub fn fetch_optional<'e, E: 'e>(
        self,
        executor: &'e mut E,
    ) -> BoxFuture<'e, crate::Result<Option<O>>>
    where
        E: Executor<Backend = DB>,
        DB: 'e,
        P: 'e,
        O: 'e,
    {
        executor.fetch_optional(self.query, self.params)
    }

    #[inline]
    pub fn fetch_one<'e, E: 'e>(self, executor: &'e mut E) -> BoxFuture<'e, crate::Result<O>>
    where
        E: Executor<Backend = DB>,
        DB: 'e,
        P: 'e,
        O: 'e,
    {
        executor.fetch_one(self.query, self.params)
    }
}
