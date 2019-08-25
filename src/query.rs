use crate::{
    backend::Backend, error::Error, executor::Executor, row::FromSqlRow, serialize::ToSql,
    types::HasSqlType,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait QueryParameters: Send {
    type Backend: Backend;

    fn new() -> Self
    where
        Self: Sized;

    fn bind<T>(&mut self, value: T)
    where
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>;
}

pub trait IntoQueryParameters<DB>
where
    DB: Backend,
{
    fn into(self) -> DB::QueryParameters;
}

#[allow(unused)]
macro_rules! impl_into_query_parameters {

    ($( ($idx:tt) -> $T:ident );+;) => {
        impl<$($T,)+ DB> IntoQueryParameters<DB> for ($($T,)+)
        where
            DB: Backend,
            $(DB: crate::types::HasSqlType<$T>,)+
            $($T: crate::serialize::ToSql<DB>,)+
        {
            fn into(self) -> DB::QueryParameters {
                let mut params = DB::QueryParameters::new();
                $(params.bind(self.$idx);)+
                params
            }
        }
    };
}

impl<DB> IntoQueryParameters<DB> for ()
where
    DB: Backend,
{
    fn into(self) -> DB::QueryParameters {
        DB::QueryParameters::new()
    }
}

impl_into_query_parameters!(
    (0) -> T1;
);

impl_into_query_parameters!(
    (0) -> T1;
    (1) -> T2;
);

impl_into_query_parameters!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
);

impl_into_query_parameters!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
);

impl_into_query_parameters!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
);

impl_into_query_parameters!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
);

pub struct SqlQuery<'q, DB>
where
    DB: Backend,
{
    query: &'q str,
    params: DB::QueryParameters,
}

impl<'q, DB> SqlQuery<'q, DB>
where
    DB: Backend,
{
    #[inline]
    pub fn new(query: &'q str) -> Self {
        Self {
            query,
            params: DB::QueryParameters::new(),
        }
    }

    #[inline]
    pub fn bind<T>(mut self, value: T) -> Self
    where
        DB: HasSqlType<T>,
        T: ToSql<DB>,
    {
        self.params.bind(value);
        self
    }

    // TODO: These methods should go on a [Execute] trait (so more execut-able things can be defined)

    #[inline]
    pub fn execute<E>(self, executor: &'q E) -> BoxFuture<'q, Result<u64, Error>>
    where
        E: Executor<Backend = DB>,
    {
        executor.execute(self.query, self.params)
    }

    #[inline]
    pub fn fetch<E, T: 'q>(self, executor: &'q E) -> BoxStream<'q, Result<T, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB> + Send + Unpin,
    {
        executor.fetch(self.query, self.params)
    }

    #[inline]
    pub fn fetch_optional<E, T: 'q>(
        self,
        executor: &'q E,
    ) -> BoxFuture<'q, Result<Option<T>, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB>,
    {
        executor.fetch_optional(self.query, self.params)
    }
}

/// Construct a full SQL query using raw SQL.
#[inline]
pub fn query<'q, DB>(query: &'q str) -> SqlQuery<'q, DB>
where
    DB: Backend,
{
    SqlQuery::new(query)
}
