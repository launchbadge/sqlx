use crate::{
    describe::Describe, executor::Executor, params::QueryParameters, row::Row,
    types::HasTypeMetadata,
};
use futures_core::future::BoxFuture;

/// A database backend.
///
/// Represents a connection to the database and further provides auxillary but
/// important related traits as associated types.
///
/// This trait is not intended to be used directly.
/// Instead [sqlx::Connection] or [sqlx::Pool] should be used instead.
pub trait Backend:
    Executor<Backend = Self> + HasTypeMetadata + Send + Sync + Sized + 'static
{
    /// The concrete `QueryParameters` implementation for this backend.
    type QueryParameters: QueryParameters<Backend = Self>;

    /// The concrete `Row` implementation for this backend.
    type Row: Row<Backend = Self>;

    /// The identifier for tables; in Postgres this is an `oid` while
    /// in MariaDB/MySQL this is the qualified name of the table.
    type TableIdent;

    /// Establish a new connection to the database server.
    fn open(url: &str) -> BoxFuture<'static, crate::Result<Self>>
    where
        Self: Sized;

    /// Release resources for this database connection immediately.
    ///
    /// This method is not required to be called. A database server will
    /// eventually notice and clean up not fully closed connections.
    fn close(self) -> BoxFuture<'static, crate::Result<()>>;
}
