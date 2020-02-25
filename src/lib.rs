#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

// Modules
pub use sqlx_core::{arguments, describe, error, pool, row, types};

// Types
pub use sqlx_core::{
    Connect, Connection, Database, Error, Executor, FromRow, Pool, Query, QueryAs, Result, Row,
    Transaction,
};

// Functions
pub use sqlx_core::{query, query_as};

#[doc(hidden)]
pub use sqlx_core::query_as_mapped;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub use sqlx_core::mysql::{self, MySql, MySqlConnection, MySqlPool};

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use sqlx_core::postgres::{self, PgConnection, PgPool, Postgres};

#[cfg(feature = "macros")]
#[doc(hidden)]
pub extern crate sqlx_macros;

#[cfg(feature = "macros")]
mod macros;

// macro support
#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod ty_cons;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod result_ext;

pub mod encode;

pub mod decode;
