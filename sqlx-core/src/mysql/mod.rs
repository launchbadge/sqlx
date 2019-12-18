mod backend;
mod connection;
mod error;
mod establish;
mod executor;
mod io;
mod protocol;
mod query;
mod row;
pub mod types;

use self::connection::Connection as RawConnection;
use crate::cache::StatementCache;
use futures_core::future::BoxFuture;
use crate::Backend;

/// Backend for MySQL.
pub enum MySql {}

impl MySql {
    /// An alias for [Backend::connect()](../trait.Backend.html#method.connect)
    pub async fn connect(url: &str) -> crate::Result<Connection> {
        <Self as Backend>::connect(url).await
    }
}

pub struct Connection {
    conn: RawConnection,
    cache: StatementCache<u32>,
}

impl crate::Connection for Connection {
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(self.conn.close())
    }
}
