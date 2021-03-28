//! [PostgreSQL] database driver.
//!
//! [PostgreSQL]: https://www.postgresql.org/
//!

use crate::DefaultRuntime;

pub type PgConnection<Rt = DefaultRuntime> = sqlx_postgres::PgConnection<Rt>;

pub use sqlx_postgres::{
    types, PgColumn, PgConnectOptions, PgQueryResult, PgRawValue, PgRawValueFormat, PgRow,
    PgTypeId, Postgres,
};
