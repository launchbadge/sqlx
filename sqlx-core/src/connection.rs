use crate::database::Database;
use crate::describe::Describe;
use crate::executor::Executor;
use crate::url::Url;
use futures_core::future::BoxFuture;
use futures_util::TryFutureExt;
use std::convert::TryInto;

/// Represents a single database connection rather than a pool of database connections.
///
/// Prefer running queries from [Pool] unless there is a specific need for a single, continuous
/// connection.
pub trait Connection
where
    Self: Send + 'static,
{
    type Database: Database;

    /// Close this database connection.
    fn close(self) -> BoxFuture<'static, crate::Result<()>>;

    /// Verifies a connection to the database is still alive.
    fn ping(&mut self) -> BoxFuture<crate::Result<()>>
    where
        for<'a> &'a mut Self: Executor<'a>,
    {
        Box::pin((&mut *self).execute("SELECT 1").map_ok(|_| ()))
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>> {
        todo!("make this a required function");
    }
}

/// Represents a type that can directly establish a new connection.
pub trait Connect: Connection {
    /// Establish a new database connection.
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<Self>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized;
}
