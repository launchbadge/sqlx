use std::convert::TryInto;

use futures_core::future::BoxFuture;

use crate::executor::Executor;
use crate::pool::{Pool, PoolConnection};
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

mod internal {
    pub enum MaybeOwnedConnection<'c, C>
    where
        C: super::Connect,
    {
        Borrowed(&'c mut C),
        Owned(super::PoolConnection<C>),
    }

    pub enum ConnectionSource<'c, C>
    where
        C: super::Connect,
    {
        Connection(MaybeOwnedConnection<'c, C>),
        Pool(super::Pool<C>),
    }
}

pub(crate) use self::internal::{ConnectionSource, MaybeOwnedConnection};

impl<'c, C> ConnectionSource<'c, C>
where
    C: Connect,
{
    pub(crate) async fn resolve_by_ref(&mut self) -> crate::Result<&'_ mut C> {
        if let ConnectionSource::Pool(pool) = self {
            *self =
                ConnectionSource::Connection(MaybeOwnedConnection::Owned(pool.acquire().await?));
        }

        Ok(match self {
            ConnectionSource::Connection(conn) => match conn {
                MaybeOwnedConnection::Borrowed(conn) => &mut *conn,
                MaybeOwnedConnection::Owned(ref mut conn) => conn,
            },
            ConnectionSource::Pool(_) => unreachable!(),
        })
    }
}

impl<'c, C> From<&'c mut C> for MaybeOwnedConnection<'c, C>
where
    C: Connect,
{
    fn from(conn: &'c mut C) -> Self {
        MaybeOwnedConnection::Borrowed(conn)
    }
}

impl<'c, C> From<PoolConnection<C>> for MaybeOwnedConnection<'c, C>
where
    C: Connect,
{
    fn from(conn: PoolConnection<C>) -> Self {
        MaybeOwnedConnection::Owned(conn)
    }
}
