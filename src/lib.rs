#![allow(dead_code)]

// Modules
pub use sqlx_core::{arguments, decode, describe, encode, error, pool, row, types};

// Types
pub use sqlx_core::{
    Connect, Connection, Database, Error, Executor, FromRow, Pool, Query, QueryAs, Result, Row,
};

// Functions
pub use sqlx_core::{query, query_as};

#[doc(hidden)]
pub use sqlx_core::query_as_mapped;

#[cfg(feature = "mysql")]
pub use sqlx_core::mysql::{self, MySql, MySqlConnection, MySqlPool};

#[cfg(feature = "postgres")]
pub use sqlx_core::postgres::{self, PgConnection, PgPool, Postgres};

#[cfg(feature = "macros")]
#[doc(hidden)]
pub extern crate sqlx_macros;

#[cfg(feature = "macros")]
#[macro_export]
mod macros;

// macro support
#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod ty_cons;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod result_ext;
