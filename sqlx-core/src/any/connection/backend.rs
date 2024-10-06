use crate::any::{Any, AnyArguments, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo};
use crate::describe::Describe;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use std::fmt::Debug;

pub trait AnyConnectionBackend: std::any::Any + Debug + Send + 'static {
    /// The backend name.
    fn name(&self) -> &str;

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database backend
    /// will be faster at cleaning up resources.
    fn close(self: Box<Self>) -> BoxFuture<'static, crate::Result<()>>;

    /// Immediately close the connection without sending a graceful shutdown.
    ///
    /// This should still at least send a TCP `FIN` frame to let the server know we're dying.
    #[doc(hidden)]
    fn close_hard(self: Box<Self>) -> BoxFuture<'static, crate::Result<()>>;

    /// Checks if a connection to the database is still valid.
    fn ping(&mut self) -> BoxFuture<'_, crate::Result<()>>;

    /// Begin a new transaction or establish a savepoint within the active transaction.
    fn begin(&mut self) -> BoxFuture<'_, crate::Result<()>>;

    fn commit(&mut self) -> BoxFuture<'_, crate::Result<()>>;

    fn rollback(&mut self) -> BoxFuture<'_, crate::Result<()>>;

    fn start_rollback(&mut self);

    /// The number of statements currently cached in the connection.
    fn cached_statements_size(&self) -> usize {
        0
    }

    /// Removes all statements from the cache, closing them on the server if
    /// needed.
    fn clear_cached_statements(&mut self) -> BoxFuture<'_, crate::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    /// Forward to [`Connection::shrink_buffers()`].
    ///
    /// [`Connection::shrink_buffers()`]: method@crate::connection::Connection::shrink_buffers
    fn shrink_buffers(&mut self);

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, crate::Result<()>>;

    #[doc(hidden)]
    fn should_flush(&self) -> bool;

    #[cfg(feature = "migrate")]
    fn as_migrate(&mut self) -> crate::Result<&mut (dyn crate::migrate::Migrate + Send + 'static)> {
        Err(crate::Error::Configuration(
            format!(
                "{} driver does not support migrations or `migrate` feature was not enabled",
                self.name()
            )
            .into(),
        ))
    }

    fn fetch_many<'q>(
        &'q mut self,
        query: &'q str,
        persistent: bool,
        arguments: Option<AnyArguments<'q>>,
    ) -> BoxStream<'q, crate::Result<Either<AnyQueryResult, AnyRow>>>;

    fn fetch_optional<'q>(
        &'q mut self,
        query: &'q str,
        persistent: bool,
        arguments: Option<AnyArguments<'q>>,
    ) -> BoxFuture<'q, crate::Result<Option<AnyRow>>>;

    fn prepare_with<'c, 'q: 'c>(
        &'c mut self,
        sql: &'q str,
        parameters: &[AnyTypeInfo],
    ) -> BoxFuture<'c, crate::Result<AnyStatement<'q>>>;

    fn describe<'q>(&'q mut self, sql: &'q str) -> BoxFuture<'q, crate::Result<Describe<Any>>>;
}
