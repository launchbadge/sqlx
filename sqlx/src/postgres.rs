//! [PostgreSQL] database driver.
//!

mod connection;
mod options;

// #[cfg(feature = "blocking")]
// mod blocking;

//
// these types are wrapped instead of re-exported
// this is to provide runtime-specialized inherent methods by taking advantage
// of through crate-local negative reasoning
pub use connection::PgConnection;
pub use options::PgConnectOptions;
//
// re-export the remaining types from the driver
pub use sqlx_postgres::{
    types, PgColumn, PgQueryResult, PgRawValue, PgRawValueFormat, PgRow, PgTypeId, Postgres,
};
