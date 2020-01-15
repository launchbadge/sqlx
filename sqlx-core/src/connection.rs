use crate::executor::Executor;
use crate::url::Url;
use futures_core::future::BoxFuture;
use futures_util::TryFutureExt;
use std::convert::TryInto;

/// Represents a single database connection rather than a pool of database connections.
///
/// Prefer running queries from [Pool] unless there is a specific need for a single, continuous
/// connection.
pub trait Connection: Executor + Send + 'static {
    /// Close this database connection.
    fn close(self) -> BoxFuture<'static, crate::Result<()>>;

    /// Verifies a connection to the database is still alive.
    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        Box::pin(self.execute("SELECT 1", Default::default()).map_ok(|_| ()))
    }
}

/// Represents a type that can directly establish a new connection.
pub trait Connect {
    type Connection: Connection;

    /// Establish a new database connection.
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<Self::Connection>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized;
}
