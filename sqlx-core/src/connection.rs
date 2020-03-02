use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;
use futures_util::TryFutureExt;

use crate::database::Database;
use crate::describe::Describe;
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
        Empty,
        Connection(MaybeOwnedConnection<'c, C>),
        Pool(super::Pool<C>),
    }
}

pub(crate) use self::internal::{ConnectionSource, MaybeOwnedConnection};

impl<'c, C> MaybeOwnedConnection<'c, C>
where
    C: Connect,
{
    pub(crate) fn borrow(&mut self) -> &'_ mut C {
        match self {
            MaybeOwnedConnection::Borrowed(conn) => &mut *conn,
            MaybeOwnedConnection::Owned(ref mut conn) => conn,
        }
    }
}

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
            ConnectionSource::Empty => panic!("`PgCursor` must not be used after being polled"),
            ConnectionSource::Connection(conn) => conn.borrow(),
            ConnectionSource::Pool(_) => unreachable!(),
        })
    }

    pub(crate) async fn resolve(mut self) -> crate::Result<MaybeOwnedConnection<'c, C>> {
        if let ConnectionSource::Pool(pool) = self {
            self = ConnectionSource::Connection(MaybeOwnedConnection::Owned(pool.acquire().await?));
        }

        Ok(self.into_connection())
    }

    pub(crate) fn into_connection(self) -> MaybeOwnedConnection<'c, C> {
        match self {
            ConnectionSource::Connection(conn) => conn,
            ConnectionSource::Empty | ConnectionSource::Pool(_) => {
                panic!("`PgCursor` must not be used after being polled");
            }
        }
    }
}

impl<C> Default for ConnectionSource<'_, C>
where
    C: Connect,
{
    fn default() -> Self {
        ConnectionSource::Empty
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

impl<'c, C> Deref for MaybeOwnedConnection<'c, C>
where
    C: Connect,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwnedConnection::Borrowed(conn) => conn,
            MaybeOwnedConnection::Owned(conn) => conn,
        }
    }
}

impl<'c, C> DerefMut for MaybeOwnedConnection<'c, C>
where
    C: Connect,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeOwnedConnection::Borrowed(conn) => conn,
            MaybeOwnedConnection::Owned(conn) => conn,
        }
    }
}
