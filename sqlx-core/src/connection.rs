use std::str::FromStr;

use futures_core::future::BoxFuture;
use futures_core::Future;

use crate::database::Database;
use crate::error::{BoxDynError, Error};
use crate::transaction::Transaction;

/// Represents a single database connection.
pub trait Connection: Send {
    type Database: Database;

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
    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database, Self>, Error>>
    where
        Self: Sized,
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
        T: Send,
        F: FnOnce(&mut <Self::Database as Database>::Connection) -> Fut + Send + 'f,
        E: From<Error> + Send,
        Fut: Future<Output = Result<T, E>> + Send,
    {
        Box::pin(async move {
            let mut tx = self.begin().await?;

            match f(tx.get_mut()).await {
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

    /// Flush any pending commands to the database.
    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>>;

    #[doc(hidden)]
    fn get_ref(&self) -> &<Self::Database as Database>::Connection;

    #[doc(hidden)]
    fn get_mut(&mut self) -> &mut <Self::Database as Database>::Connection;

    #[doc(hidden)]
    fn transaction_depth(&self) -> usize {
        // connections are not normally transactions, a zero depth implies there is no
        // active transaction
        0
    }
}

/// Represents a type that can directly establish a new connection.
pub trait Connect: Sized + Connection {
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
}
