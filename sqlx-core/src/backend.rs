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
pub trait Backend: HasTypeMetadata + Send + Sync + Sized + 'static {
    type Connection: crate::Connection<Backend = Self>;

    /// The concrete `QueryParameters` implementation for this backend.
    type QueryParameters: QueryParameters<Backend = Self>;

    /// The concrete `Row` implementation for this backend.
    type Row: Row<Backend = Self>;

    /// The identifier for tables; in Postgres this is an `oid` while
    /// in MySQL/MariaDB this is the qualified name of the table.
    type TableIdent;

    /// Establish a new connection to the database server.
    fn connect(url: &str) -> BoxFuture<'static, crate::Result<Self::Connection>>;
}
