use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;

use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::executor::Executor;

/// Generic management of database transactions.
///
/// This trait should not be used, except when implementing [`Connection`].
#[doc(hidden)]
pub trait TransactionManager {
    type Database: Database;

    /// Begin a new transaction or establish a savepoint within the active transaction.
    fn begin(
        conn: &mut <Self::Database as Database>::Connection,
        depth: usize,
    ) -> BoxFuture<Result<(), Error>>;

    /// Commit the active transaction or release the most recent savepoint.
    fn commit(
        conn: &mut <Self::Database as Database>::Connection,
        depth: usize,
    ) -> BoxFuture<Result<(), Error>>;

    /// Abort the active transaction or restore from the most recent savepoint.
    fn rollback(
        conn: &mut <Self::Database as Database>::Connection,
        depth: usize,
    ) -> BoxFuture<Result<(), Error>>;
}

/// An in-progress database transaction or savepoint.
///
/// A transaction starts with a call to [`Pool::begin`] or [`Connection::begin`].
///
/// A transaction should end with a call to [`commit`] or [`rollback`]. If neither are called
/// before the transaction goes out-of-scope, [`rollback`] is called. In other
/// words, [`rollback`] is called on `drop` if the transaction is still in-progress.
///
/// A savepoint is a special mark inside a transaction that allows all commands that are
/// executed after it was established to be rolled back, restoring the transaction state to
/// what it was at the time of the savepoint.
///
/// [`Connection::begin`]: struct.Connection.html#method.begin
/// [`Pool::begin`]: struct.Pool.html#method.begin
/// [`commit`]: #method.commit
/// [`rollback`]: #method.rollback
pub struct Transaction<'c, C: Connection + ?Sized> {
    connection: &'c mut C,

    // the depth of ~this~ transaction, depth directly equates to how many times [`begin()`]
    // was called without a corresponding [`commit()`] or [`rollback()`]
    depth: usize,
}

impl<'c, DB, C> Transaction<'c, C>
where
    DB: Database,
    C: ?Sized + Connection<Database = DB>,
{
    pub(crate) fn begin(conn: &'c mut C) -> BoxFuture<'c, Result<Self, Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth();

            DB::TransactionManager::begin(conn.get_mut(), depth).await?;

            Ok(Self {
                depth: depth + 1,
                connection: conn,
            })
        })
    }

    /// Commits this transaction or savepoint.
    pub async fn commit(self) -> Result<(), Error> {
        DB::TransactionManager::commit(self.connection.get_mut(), self.depth).await
    }

    /// Aborts this transaction or savepoint.
    pub async fn rollback(self) -> Result<(), Error> {
        DB::TransactionManager::rollback(self.connection.get_mut(), self.depth).await
    }
}

impl<'c, C: Connection + ?Sized> Connection for Transaction<'c, C> {
    type Database = C::Database;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        unimplemented!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.connection.ping()
    }

    #[doc(hidden)]
    fn get_ref(&self) -> &<Self::Database as Database>::Connection {
        self.connection.get_ref()
    }

    #[doc(hidden)]
    fn get_mut(&mut self) -> &mut <Self::Database as Database>::Connection {
        self.connection.get_mut()
    }

    #[doc(hidden)]
    fn transaction_depth(&self) -> usize {
        self.depth
    }
}

impl<DB, C> Deref for Transaction<'_, C>
where
    DB: Database,
    C: ?Sized + Connection<Database = DB>,
{
    type Target = DB::Connection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<DB, C> DerefMut for Transaction<'_, C>
where
    DB: Database,
    C: ?Sized + Connection<Database = DB>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<C: Connection + ?Sized> Drop for Transaction<'_, C> {
    fn drop(&mut self) {
        unimplemented!()
    }
}

#[allow(dead_code)]
pub(crate) async fn begin_ansi_transaction<'c, C>(
    conn: &'c mut C,
    index: usize,
) -> Result<(), Error>
where
    &'c mut C: Executor<'c>,
{
    if index == 0 {
        conn.execute("BEGIN").await?;
    } else {
        conn.execute(&*format!("SAVEPOINT _sqlx_savepoint_{}", index))
            .await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn commit_ansi_transaction<'c, C>(
    conn: &'c mut C,
    index: usize,
) -> Result<(), Error>
where
    &'c mut C: Executor<'c>,
{
    if index == 1 {
        conn.execute("COMMIT").await?;
    } else {
        conn.execute(&*format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", index))
            .await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn rollback_ansi_transaction<'c, C>(
    conn: &'c mut C,
    index: usize,
) -> Result<(), Error>
where
    &'c mut C: Executor<'c>,
{
    if index == 1 {
        conn.execute("ROLLBACK").await?;
    } else {
        conn.execute(&*format!("ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}", index))
            .await?;
    }

    Ok(())
}
