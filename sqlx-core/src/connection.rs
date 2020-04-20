use std::convert::TryInto;

use futures_core::future::BoxFuture;

use crate::database::Database;
use crate::error::{BoxDynError, Error};
use std::str::FromStr;

// TODO: Connection::begin
// TODO: Connection::transaction

/// Represents a single database connection.
pub trait Connection: Send + 'static {
    type Database: Database;

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database backend
    /// will be faster at cleaning up resources.
    fn close(self) -> BoxFuture<'static, Result<(), Error>>;

    /// Checks if a connection to the database is still valid.
    fn ping(&mut self) -> BoxFuture<Result<(), Error>>;
}

/// Represents a type that can directly establish a new connection.
pub trait Connect: Sized + Connection {
    type Options: FromStr<Err = BoxDynError> + Send + Sync;

    /// Establish a new database connection.
    #[inline]
    fn connect(url: &str) -> BoxFuture<'static, Result<Self, Error>> {
        let options = url.parse().map_err(Error::ParseConnectOptions);

        Box::pin(async move { Ok(Self::connect_with(&options?).await?) })
    }

    /// Establish a new database connection.
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>>;
}
