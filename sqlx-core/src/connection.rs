use std::str::FromStr;

use futures_core::future::BoxFuture;
use futures_core::Future;

use crate::database::{Database, HasStatementCache};
use crate::error::{BoxDynError, Error};
use crate::transaction::Transaction;

/// Represents a single database connection.
pub trait Connection: Send {
    type Database: Database;
    type Options: FromStr<Err = BoxDynError> + Send + Sync + 'static;

    /// Establish a new database connection.
    ///
    /// A value of `Options` is parsed from the provided connection string. This parsing
    /// is database-specific.
    #[inline]
    fn connect(url: &str) -> BoxFuture<'static, Result<Self, Error>> {
        let options = url.parse().map_err(Error::ParseConnectOptions);

        Box::pin(async move { Ok(Self::connect_with(&options?).await?) })
    }

    /// Establish a new database connection with the provided options.
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>>;

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database backend
    /// will be faster at cleaning up resources.
    fn close(self) -> BoxFuture<'static, Result<(), Error>>;

    /// Checks if a connection to the database is still valid.
    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>>;

    /// Begin a new transaction or establish a savepoint within the active transaction.
    ///
    /// Returns a [`Transaction`] for controlling and tracking the new transaction.
    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
        Self::Database: Database<Connection = Self>,
    {
        Transaction::begin(self)
    }

    /// Execute the function inside a transaction.
    ///
    /// If the function returns an error, the transaction will be rolled back. If it does not
    /// return an error, the transaction will be committed.
    fn transaction<'c: 'f, 'f, T, E, F, Fut>(&'c mut self, f: F) -> BoxFuture<'f, Result<T, E>>
    where
        Self: Sized,
        Self::Database: Database<Connection = Self>,
        T: Send,
        F: FnOnce(&mut Self) -> Fut + Send + 'f,
        E: From<Error> + Send,
        Fut: Future<Output = Result<T, E>> + Send,
    {
        Box::pin(async move {
            let mut tx = self.begin().await?;

            match f(&mut *tx).await {
                Ok(r) => {
                    // no error occurred, commit the transaction
                    tx.commit().await?;

                    Ok(r)
                }

                Err(e) => {
                    // an error occurred, rollback the transaction
                    tx.rollback().await?;

                    Err(e)
                }
            }
        })
    }

    /// The number of statements currently cached in the connection.
    fn cached_statements_size(&self) -> usize
    where
        Self::Database: HasStatementCache,
    {
        0
    }

    /// Removes all statements from the cache, closing them on the server if
    /// needed.
    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>>
    where
        Self::Database: HasStatementCache,
    {
        Box::pin(async move { Ok(()) })
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>>;

    #[doc(hidden)]
    fn should_flush(&self) -> bool;

    #[doc(hidden)]
    fn transaction_depth(&self) -> usize;
}
