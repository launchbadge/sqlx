use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;
use futures_util::{future, FutureExt};

use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::ext::maybe_owned::MaybeOwned;

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
    ) -> BoxFuture<'_, Result<(), Error>>;

    /// Commit the active transaction or release the most recent savepoint.
    fn commit(
        conn: &mut <Self::Database as Database>::Connection,
        depth: usize,
    ) -> BoxFuture<'_, Result<(), Error>>;

    /// Abort the active transaction or restore from the most recent savepoint.
    fn rollback(
        conn: &mut <Self::Database as Database>::Connection,
        depth: usize,
    ) -> BoxFuture<'_, Result<(), Error>>;

    /// Starts to abort the active transaction or restore from the most recent snapshot.
    fn start_rollback(conn: &mut <Self::Database as Database>::Connection, depth: usize);
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
pub struct Transaction<'c, DB, C = <DB as Database>::Connection>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    connection: MaybeOwned<'c, C>,

    // the depth of ~this~ transaction, depth directly equates to how many times [`begin()`]
    // was called without a corresponding [`commit()`] or [`rollback()`]
    depth: usize,
}

impl<'c, DB, C> Transaction<'c, DB, C>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    pub(crate) fn begin(conn: impl Into<MaybeOwned<'c, C>>) -> BoxFuture<'c, Result<Self, Error>> {
        let mut conn = conn.into();

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
    pub async fn commit(mut self) -> Result<(), Error> {
        DB::TransactionManager::commit(self.connection.get_mut(), self.depth).await
    }

    /// Aborts this transaction or savepoint.
    pub async fn rollback(mut self) -> Result<(), Error> {
        DB::TransactionManager::rollback(self.connection.get_mut(), self.depth).await
    }
}

impl<'c, DB, C> Connection for Transaction<'c, DB, C>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    type Database = C::Database;

    // equivalent to dropping the transaction
    // as this is bound to 'static, there is nothing more we can do here
    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        future::ok(()).boxed()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.connection.ping()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.get_mut().flush()
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

// NOTE: required due to lack of lazy normalization
#[allow(unused_macros)]
macro_rules! impl_executor_for_transaction {
    ($DB:ident, $Row:ident) => {
        impl<'c, 't, C: Sized> crate::executor::Executor<'t>
            for &'t mut crate::transaction::Transaction<'c, $DB, C>
        where
            C: crate::connection::Connection<Database = $DB>,
        {
            type Database = C::Database;

            fn fetch_many<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> futures_core::stream::BoxStream<
                'e,
                Result<either::Either<u64, $Row>, crate::error::Error>,
            >
            where
                't: 'e,
                E: crate::executor::Execute<'q, Self::Database>,
            {
                crate::connection::Connection::get_mut(self).fetch_many(query)
            }

            fn fetch_optional<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> futures_core::future::BoxFuture<'e, Result<Option<$Row>, crate::error::Error>>
            where
                't: 'e,
                E: crate::executor::Execute<'q, Self::Database>,
            {
                crate::connection::Connection::get_mut(self).fetch_optional(query)
            }

            #[doc(hidden)]
            fn describe<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> futures_core::future::BoxFuture<
                'e,
                Result<crate::describe::Describe<Self::Database>, crate::error::Error>,
            >
            where
                't: 'e,
                E: crate::executor::Execute<'q, Self::Database>,
            {
                crate::connection::Connection::get_mut(self).describe(query)
            }
        }
    };
}

impl<'c, DB, C> Debug for Transaction<'c, DB, C>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Show the full type <..<..<..
        f.debug_struct("Transaction").finish()
    }
}

impl<'c, DB, C> Deref for Transaction<'c, DB, C>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    type Target = <C::Database as Database>::Connection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<'c, DB, C> DerefMut for Transaction<'c, DB, C>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<'c, DB, C> Drop for Transaction<'c, DB, C>
where
    DB: Database,
    C: Sized + Connection<Database = DB>,
{
    fn drop(&mut self) {
        // starts a rollback operation
        // what this does depends on the database but generally this means we queue a rollback
        // operation that will happen on the next asynchronous invocation of the underlying
        // connection (including if the connection is returned to a pool)
        <C::Database as Database>::TransactionManager::start_rollback(
            self.connection.get_mut(),
            self.depth,
        );
    }
}

#[allow(dead_code)]
pub(crate) fn begin_ansi_transaction_sql(index: usize) -> Cow<'static, str> {
    if index == 0 {
        Cow::Borrowed("BEGIN")
    } else {
        Cow::Owned(format!("SAVEPOINT _sqlx_savepoint_{}", index))
    }
}

#[allow(dead_code)]
pub(crate) fn commit_ansi_transaction_sql(index: usize) -> Cow<'static, str> {
    if index == 1 {
        Cow::Borrowed("COMMIT")
    } else {
        Cow::Owned(format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", index))
    }
}

#[allow(dead_code)]
pub(crate) fn rollback_ansi_transaction_sql(index: usize) -> Cow<'static, str> {
    if index == 1 {
        Cow::Borrowed("ROLLBACK")
    } else {
        Cow::Owned(format!("ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}", index))
    }
}
