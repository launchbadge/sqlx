#![cfg_attr(docsrs, feature(doc_cfg))]

pub use sqlx_core::arguments::{Arguments, IntoArguments};
pub use sqlx_core::connection::{Connect, Connection};
pub use sqlx_core::database::{self, Database};
pub use sqlx_core::executor::{Execute, Executor};
pub use sqlx_core::from_row::FromRow;
pub use sqlx_core::pool::{self, Pool};
pub use sqlx_core::query::{query, query_with};
pub use sqlx_core::query_as::{query_as, query_as_with};
pub use sqlx_core::query_scalar::{query_scalar, query_scalar_with};
pub use sqlx_core::row::{ColumnIndex, Row};
pub use sqlx_core::transaction::{Transaction, TransactionManager};
pub use sqlx_core::type_info::TypeInfo;
pub use sqlx_core::types::Type;
pub use sqlx_core::value::{Value, ValueRef};

#[doc(hidden)]
pub use sqlx_core::describe;

#[doc(inline)]
pub use sqlx_core::error::{self, Error, Result};

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub use sqlx_core::mysql::{self, MySql, MySqlConnection, MySqlPool};

#[cfg(feature = "mssql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mssql")))]
pub use sqlx_core::mssql::{self, Mssql, MssqlConnection, MssqlPool};

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use sqlx_core::postgres::{self, PgConnection, PgPool, Postgres};

#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub use sqlx_core::sqlite::{self, Sqlite, SqliteConnection, SqlitePool};

#[cfg(feature = "macros")]
#[doc(hidden)]
pub extern crate sqlx_macros;

// derives
#[cfg(feature = "macros")]
#[doc(hidden)]
pub use sqlx_macros::{FromRow, Type};

#[cfg(feature = "macros")]
mod macros;

// macro support
#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod ty_match;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod result_ext;

pub mod types {
    pub use sqlx_core::types::*;

    #[cfg(feature = "macros")]
    #[doc(hidden)]
    pub use sqlx_macros::Type;
}

/// Provides [`Encode`](encode/trait.Encode.html) for encoding values for the database.
pub mod encode {
    pub use sqlx_core::encode::{Encode, IsNull};

    #[cfg(feature = "macros")]
    #[doc(hidden)]
    pub use sqlx_macros::Encode;
}

/// Provides [`Decode`](decode/trait.Decode.html) for decoding values from the database.
pub mod decode {
    pub use sqlx_core::decode::Decode;

    #[cfg(feature = "macros")]
    #[doc(hidden)]
    pub use sqlx_macros::Decode;
}

pub mod query {
    pub use sqlx_core::query::{query, query_with, Map, Query};
    pub use sqlx_core::query_as::{query_as, query_as_with, QueryAs};
    pub use sqlx_core::query_scalar::{query_scalar, query_scalar_with, QueryScalar};
}

/// Convenience re-export of common traits.
pub mod prelude {
    pub use super::Connect;
    pub use super::Connection;
    pub use super::Executor;
    pub use super::FromRow;
    pub use super::IntoArguments;
    pub use super::Row;
    pub use super::Type;
}
