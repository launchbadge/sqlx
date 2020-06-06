//! **SQLite** database driver.
//!
//! Query example:
//!
//!```rust,ignore
//! use sqlx::sqlite::SqlitePool;
//!
//! let pool = SqlitePool::builder()
//!     .max_size(3)
//!     .build("sqlite://tests/sqlite/sqlite.db")
//!     .await?;
//!
//! let row: (String,) = sqlx::query_as("SELECT text FROM tweet")
//!     .fetch_one(&pool)
//!     .await?;
//!
//! assert_eq!(row.0, "Hi!");
//! ```
//!
//! Custom decoder:
//!
//! ```rust,ignore
//! use sqlx::decode::Decode;
//! use sqlx::sqlite::{Sqlite, SqliteTypeInfo, SqliteValue};
//!
//! pub struct HexBlob(pub String);
//!
//! impl sqlx::Type<Sqlite> for HexBlob {
//!     fn type_info() -> SqliteTypeInfo {
//!         <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
//!     }
//! }
//!
//! impl<'de> Decode<'de, Sqlite> for HexBlob {
//!     fn decode(value: SqliteValue<'de>) -> sqlx::Result<Self> {
//!         let blob = <Vec<u8> as Decode<Sqlite>>::decode(value)?;
//!         let hex = blob
//!             .into_iter()
//!             .map(|b| format!("{:x}", b))
//!             .collect::<Vec<String>>()
//!             .join("");
//!         Ok(HexBlob(hex))
//!     }
//! }
//! ```

// SQLite is a C library. All interactions require FFI which is unsafe.
// All unsafe blocks should have comments pointing to SQLite docs and ensuring that we maintain
// invariants.
#![allow(unsafe_code)]

mod arguments;
mod connection;
mod database;
mod error;
mod options;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

pub use arguments::{SqliteArgumentValue, SqliteArguments};
pub use connection::SqliteConnection;
pub use database::Sqlite;
pub use error::SqliteError;
pub use options::SqliteConnectOptions;
pub use row::SqliteRow;
pub use transaction::SqliteTransactionManager;
pub use type_info::SqliteTypeInfo;
pub use value::{SqliteValue, SqliteValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for SQLite.
pub type SqlitePool = crate::pool::Pool<Sqlite>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(SqliteArguments<'q>);
impl_executor_for_pool_connection!(Sqlite, SqliteConnection, SqliteRow);
impl_executor_for_transaction!(Sqlite, SqliteRow);
