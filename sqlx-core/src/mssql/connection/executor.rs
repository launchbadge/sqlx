use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::mssql::{MsSql, MsSqlConnection, MsSqlRow};

impl<'c> Executor<'c> for &'c mut MsSqlConnection {
    type Database = MsSql;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<u64, MsSqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        unimplemented!()
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<MsSqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        unimplemented!()
    }

    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        unimplemented!()
    }
}
