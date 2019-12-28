//! **Postgres** database and connection types.

mod arguments;
mod connection;
mod database;
mod error;
mod executor;
mod protocol;
mod row;
mod types;

pub use database::Postgres;

pub use arguments::PgArguments;

pub use connection::PgConnection;

pub use error::PgError;

pub use row::PgRow;

/// An alias for [`Pool`], specialized for **Postgres**.
pub type PgPool = super::Pool<Postgres>;
