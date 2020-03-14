use std::convert::TryInto;

use futures_core::future::BoxFuture;

use crate::executor::Executor;
use crate::maybe_owned::MaybeOwned;
use crate::pool::{Pool, PoolConnection};
use crate::transaction::Transaction;
use crate::url::Url;

/// Represents a single database connection rather than a pool of database connections.
///
/// Prefer running queries from [Pool] unless there is a specific need for a single, continuous
/// connection.
pub trait Connection
where
    Self: Send + 'static,
    Self: Executor,
{
    /// Starts a transaction.
    ///
    /// Returns [`Transaction`](struct.Transaction.html).
    fn begin(self) -> BoxFuture<'static, crate::Result<Transaction<Self>>>
    where
        Self: Sized,
    {
        Box::pin(Transaction::new(0, self))
    }

    /// Close this database connection.
    fn close(self) -> BoxFuture<'static, crate::Result<()>>;

    /// Verifies a connection to the database is still alive.
    fn ping(&mut self) -> BoxFuture<crate::Result<()>>;
}

/// Represents a type that can directly establish a new connection.
pub trait Connect: Connection {
    /// Establish a new database connection.
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<Self>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized;
}

pub(crate) enum ConnectionSource<'c, C>
where
    C: Connect,
{
    Connection(MaybeOwned<PoolConnection<C>, &'c mut C>),

    #[allow(dead_code)]
    Pool(Pool<C>),
}

impl<'c, C> ConnectionSource<'c, C>
where
    C: Connect,
{
    #[allow(dead_code)]
    pub(crate) async fn resolve(&mut self) -> crate::Result<&'_ mut C> {
        if let ConnectionSource::Pool(pool) = self {
            *self = ConnectionSource::Connection(MaybeOwned::Owned(pool.acquire().await?));
        }

        Ok(match self {
            ConnectionSource::Connection(conn) => match conn {
                MaybeOwned::Borrowed(conn) => &mut *conn,
                MaybeOwned::Owned(ref mut conn) => conn,
            },
            ConnectionSource::Pool(_) => unreachable!(),
        })
    }
}
