use futures_core::future::BoxFuture;

use crate::error::Error;

/// A connection that is capable of caching prepared statements.
pub trait CachingConnection: Send {
    /// The number of statements currently cached in the connection.
    fn cached_statements_count(&self) -> usize;

    /// Removes all statements from the cache, closing them on the server if
    /// needed.
    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>>;
}
