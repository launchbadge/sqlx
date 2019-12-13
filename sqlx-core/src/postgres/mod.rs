use crate::postgres::connection::PostgresConn;
use crate::cache::StatementCache;

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

pub struct Postgres {
    conn: PostgresConn,
    statements: StatementCache<u64>,
    next_id: u64,
}
