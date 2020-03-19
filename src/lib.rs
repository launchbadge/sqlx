#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

pub use sqlx_core::arguments;
pub use sqlx_core::connection::{Connect, Connection};
pub use sqlx_core::cursor::Cursor;
pub use sqlx_core::database::{self, Database};
pub use sqlx_core::describe;
pub use sqlx_core::executor::{self, Execute, Executor};
pub use sqlx_core::pool::{self, Pool};
pub use sqlx_core::query::{self, query, Query};
pub use sqlx_core::query_as::{query_as, QueryAs};
pub use sqlx_core::row::{self, FromRow, Row};
pub use sqlx_core::transaction::Transaction;

#[doc(inline)]
pub use sqlx_core::types::{self, Type};

#[doc(inline)]
pub use sqlx_core::error::{self, Error, Result};

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub use sqlx_core::mysql::{self, MySql, MySqlConnection, MySqlPool};

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use sqlx_core::postgres::{self, PgConnection, PgPool, Postgres};

#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub use sqlx_core::sqlite::{self, Sqlite, SqliteConnection, SqlitePool};

#[cfg(feature = "macros")]
#[doc(hidden)]
pub extern crate sqlx_macros;

#[cfg(feature = "macros")]
pub use sqlx_macros::Type;

#[cfg(feature = "macros")]
mod macros;

// macro support
#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod ty_match;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod result_ext;

pub mod encode {
    pub use sqlx_core::encode::{Encode, IsNull};

    #[cfg(feature = "macros")]
    pub use sqlx_macros::Encode;
}

pub mod decode {
    pub use sqlx_core::decode::Decode;

    #[cfg(feature = "macros")]
    pub use sqlx_macros::Decode;
}

pub mod prelude {
    pub use super::Connect;
    pub use super::Connection;
    pub use super::Cursor;
    pub use super::Executor;
    pub use super::FromRow;
    pub use super::Row;

    #[cfg(feature = "postgres")]
    pub use super::postgres::PgQueryAs;

    #[cfg(feature = "mysql")]
    pub use super::mysql::MySqlQueryAs;

    #[cfg(feature = "sqlite")]
    pub use super::sqlite::SqliteQueryAs;
}
