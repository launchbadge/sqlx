//! Contains the `Connection` and `Connect` traits.

use std::convert::TryInto;

use futures_core::future::BoxFuture;

use crate::executor::Executor;
use crate::pool::{Pool, PoolConnection};
use crate::transaction::Transaction;
use crate::url::Url;

/// Represents a single database connection rather than a pool of database connections.
///
/// Connections can be manually established outside of a [`Pool`] with [`Connect::connect`].
///
/// Prefer running queries from [`Pool`] unless there is a specific need for a single, sticky
/// connection.
pub trait Connection
where
    Self: Send + 'static,
    Self: Executor,
{
    /// Starts a new transaction.
    ///
    /// Wraps this connection in [`Transaction`] to manage the transaction lifecycle. To get the
    /// original connection back, explicitly [`commit`] or [`rollback`] and this connection will
    /// be returned.
    ///
    /// ```rust,ignore
    /// let mut tx = conn.begin().await?;
    /// // conn is now inaccessible as its wrapped in a transaction
    ///
    /// let conn = tx.commit().await?;
    /// // conn is back now and out of the transaction
    /// ```
    ///
    /// [`commit`]: crate::transaction::Transaction::commit
    /// [`rollback`]: crate::transaction::Transaction::rollback
    fn begin(self) -> BoxFuture<'static, crate::Result<Transaction<Self>>>
    where
        Self: Sized,
    {
        Box::pin(Transaction::new(0, self))
    }

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database server
    /// will be faster at cleaning up resources.
    fn close(self) -> BoxFuture<'static, crate::Result<()>>;

    /// Checks if a connection to the database is still valid.
    fn ping(&mut self) -> BoxFuture<crate::Result<()>>;
}

/// Represents a type that can directly establish a new connection.
pub trait Connect: Connection {
    /// Establish a new database connection.
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<Self>>
    where
        T: TryInto<Url, Error = url::ParseError>,
        Self: Sized;
}

#[allow(dead_code)]
pub(crate) enum ConnectionSource<'c, C>
where
    C: Connect,
{
    ConnectionRef(&'c mut C),
    Connection(C),
    PoolConnection(Pool<C>, PoolConnection<C>),
    Pool(Pool<C>),
}

impl<'c, C> ConnectionSource<'c, C>
where
    C: Connect,
{
    #[allow(dead_code)]
    pub(crate) async fn resolve(&mut self) -> crate::Result<&'_ mut C> {
        if let ConnectionSource::Pool(pool) = self {
            let conn = pool.acquire().await?;

            *self = ConnectionSource::PoolConnection(pool.clone(), conn);
        }

        Ok(match self {
            ConnectionSource::ConnectionRef(conn) => conn,
            ConnectionSource::PoolConnection(_, ref mut conn) => conn,
            ConnectionSource::Connection(ref mut conn) => conn,
            ConnectionSource::Pool(_) => unreachable!(),
        })
    }
}

impl<'c, C> From<C> for ConnectionSource<'c, C>
where
    C: Connect,
{
    fn from(connection: C) -> Self {
        ConnectionSource::Connection(connection)
    }
}

impl<'c, C> From<PoolConnection<C>> for ConnectionSource<'c, C>
where
    C: Connect,
{
    fn from(connection: PoolConnection<C>) -> Self {
        ConnectionSource::PoolConnection(Pool(connection.pool.clone()), connection)
    }
}

impl<'c, C> From<Pool<C>> for ConnectionSource<'c, C>
where
    C: Connect,
{
    fn from(pool: Pool<C>) -> Self {
        ConnectionSource::Pool(pool)
    }
}
