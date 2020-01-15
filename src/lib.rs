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

#[allow(unused_attributes)]
#[macro_export]
mod macros;

#[cfg(feature = "macros")]
#[doc(hidden)]
#[proc_macro_hack::proc_macro_hack(fake_call_site)]
#[allow(dead_code)]
pub use sqlx_macros::query as query_;

#[cfg(feature = "macros")]
#[doc(hidden)]
#[proc_macro_hack::proc_macro_hack(fake_call_site)]
#[allow(dead_code)]
pub use sqlx_macros::query_as as query_as_;

#[cfg(feature = "macros")]
#[doc(hidden)]
#[proc_macro_hack::proc_macro_hack(fake_call_site)]
#[allow(dead_code)]
pub use sqlx_macros::query_file as query_file_;

#[cfg(feature = "macros")]
#[doc(hidden)]
#[proc_macro_hack::proc_macro_hack(fake_call_site)]
#[allow(dead_code)]
pub use sqlx_macros::query_file_as as query_file_as_;

// macro support
#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod ty_cons;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod result_ext;
