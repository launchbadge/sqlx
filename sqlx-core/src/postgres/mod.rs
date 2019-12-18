use crate::postgres::connection::Connection as RawConnection;
use crate::cache::StatementCache;
use crate::{Error, Backend};
use futures_core::Future;
use futures_core::future::BoxFuture;
use std::net::SocketAddr;
use bitflags::_core::pin::Pin;

mod backend;
mod connection;
mod error;
mod executor;
mod query;
mod row;

#[cfg(not(feature = "unstable"))]
mod protocol;

#[cfg(feature = "unstable")]
pub mod protocol;

pub mod types;

/// The Postgres backend implementation.
pub enum Postgres {}

impl Postgres {
    /// Alias for [Backend::connect()](../trait.Backend.html#method.connect).
    pub async fn connect(url: &str) -> crate::Result<Connection> {
        <Self as Backend>::connect(url).await
    }
}

/// A connection to a Postgres database.
pub struct Connection {
    conn: RawConnection,
    statements: StatementCache<u64>,
    next_id: u64,
}

impl crate::Connection for Connection {
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(self.conn.terminate())
    }
}
