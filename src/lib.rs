#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub use sqlx_core::mysql::{self, MySqlConnection};

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use sqlx_core::postgres::{self, PgConnection};

#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub use sqlx_core::sqlite::{self, SqliteConnection};

/// Error and result types.
pub mod error {
    pub use sqlx_core::error::{DatabaseError, Error, Result};
}

/// Traits to describe the various database drivers.
pub mod database {
    pub use sqlx_core::database::{Database, HasArguments, HasValueRef};
}

/// Types and traits for encoding values for the database.
pub mod encode {
    pub use sqlx_core::encode::{Encode, IsNull};

    #[cfg(feature = "macros")]
    pub use sqlx_macros::Encode;
}

/// Types and traits for decoding values from the database.
pub mod decode {
    pub use sqlx_core::decode::Decode;

    #[cfg(feature = "macros")]
    pub use sqlx_macros::Decode;
}

/// Contains the [`ColumnIndex`](row::ColumnIndex), [`Row`], and [`FromRow`] traits.
pub mod row {
    pub use sqlx_core::from_row::FromRow;
    pub use sqlx_core::row::{ColumnIndex, Row};

    #[cfg(feature = "macros")]
    pub use sqlx_macros::FromRow;
}

/// Contains the [`Value`](value::Value) and [`ValueRef`](value::ValueRef) traits.
pub mod value {
    pub use sqlx_core::value::{Value, ValueRef};
}

/// Contains the return values from [`query`](query::query), [`query_as`], and [`query_scalar`].
pub mod query {
    pub use sqlx_core::query::{query, Map, Query};
    pub use sqlx_core::query_as::{query_as, QueryAs};
    pub use sqlx_core::query_scalar::{query_scalar, QueryScalar};
}

/// Convenience re-export of common traits.
pub mod prelude {
    pub use super::value::Value;
    pub use super::value::ValueRef;
    pub use super::Connect;
    pub use super::Connection;
    pub use super::Execute;
    pub use super::Executor;
    pub use super::FromRow;
    pub use super::Row;
}

/// Conversions between Rust and SQL types.
///
/// To see how each SQL type maps to a Rust type, see the corresponding `types` module for each
/// database:
///
///  * [PostgreSQL](../postgres/types/index.html)
///  * [MySQL](../mysql/types/index.html)
///  * [SQLite](../sqlite/types/index.html)
///
/// Any external types that have had [`Type`] implemented for, are re-exported in this module
/// for convenience as downstream users need to use a compatible version of the external crate
/// to take advantage of the implementation.
pub mod types {
    pub use sqlx_core::types::*;

    #[cfg(feature = "macros")]
    pub use sqlx_macros::Type;
}

pub use error::{Error, Result};
pub use query::{query, query_as, query_scalar};
pub use row::{FromRow, Row};
pub use sqlx_core::arguments::Arguments;
pub use sqlx_core::connection::{Connect, Connection};
pub use sqlx_core::executor::{Execute, Executor};
