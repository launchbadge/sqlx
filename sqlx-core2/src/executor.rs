use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::execute::Execute;
use futures_core::future::BoxFuture;

pub trait Executor<'e> {
    type Database: Database;

    /// Execute the SQL query.
    ///
    /// Returns a value of [`Done`] which signals successful query completion and provides
    /// the number of rows affected; plus, any additional database-specific information (such as
    /// the last inserted ID).
    fn execute<'x, 'q: 'x, E: 'x + Execute<'q, Self::Database>>(
        self,
        query: E,
    ) -> BoxFuture<'x, Result<u64, Error>>
    where
        'e: 'x;
}

impl<'c, C: Connection> Executor<'c> for &'c mut C {
    type Database = C::Database;

    #[inline]
    fn execute<'x, 'q: 'x, E: 'x + Execute<'q, Self::Database>>(
        self,
        query: E,
    ) -> BoxFuture<'x, Result<u64, Error>>
    where
        'c: 'x,
    {
        Connection::execute(self, query)
    }
}
