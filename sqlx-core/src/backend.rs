use crate::describe::Describe;
use crate::{params::QueryParameters, row::RawRow, types::HasTypeMetadata};
use async_trait::async_trait;
use futures_core::stream::BoxStream;

/// A database backend.
///
/// Represents a connection to the database and further provides auxillary but
/// important related traits as associated types.
///
/// This trait is not intended to be used directly.
/// Instead [sqlx::Connection] or [sqlx::Pool] should be used instead,
/// which provide concurrent access and typed retrieval of results.
#[async_trait]
pub trait Backend: HasTypeMetadata + Send + Sync + Sized + 'static {
    /// The concrete `QueryParameters` implementation for this backend.
    type QueryParameters: QueryParameters<Backend = Self>;

    /// The concrete `Row` implementation for this backend.
    type Row: RawRow<Backend = Self>;

    /// The identifier for tables; in Postgres this is an `oid` while
    /// in MariaDB/MySQL this is the qualified name of the table.
    type TableIdent;

    /// Establish a new connection to the database server.
    async fn open(url: &str) -> crate::Result<Self>
    where
        Self: Sized;

    /// Release resources for this database connection immediately.
    ///
    /// This method is not required to be called. A database server will
    /// eventually notice and clean up not fully closed connections.
    async fn close(mut self) -> crate::Result<()>;

    async fn ping(&mut self) -> crate::Result<()> {
        // TODO: Does this need to be specialized for any database backends?
        let _ = self
            .execute("SELECT 1", Self::QueryParameters::new())
            .await?;

        Ok(())
    }

    async fn describe(&mut self, query: &str) -> crate::Result<Describe<Self>>;

    async fn execute(&mut self, query: &str, params: Self::QueryParameters) -> crate::Result<u64>;

    fn fetch(
        &mut self,
        query: &str,
        params: Self::QueryParameters,
    ) -> BoxStream<'_, crate::Result<Self::Row>>;

    async fn fetch_optional(
        &mut self,
        query: &str,
        params: Self::QueryParameters,
    ) -> crate::Result<Option<Self::Row>>;
}
