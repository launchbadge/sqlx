//! **SQLite** database driver.

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
mod type_info;
pub mod types;
mod value;

pub use arguments::{SqliteArgumentValue, SqliteArguments};
pub use connection::SqliteConnection;
pub use database::Sqlite;
pub use error::SqliteError;
pub use options::SqliteConnectOptions;
pub use row::SqliteRow;
pub use type_info::SqliteTypeInfo;
pub use value::{SqliteValue, SqliteValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for SQLite.
pub type SqlitePool = crate::pool::Pool<SqliteConnection>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(SqliteArguments<'q>);
impl_executor_for_pool_connection!(Sqlite, SqliteConnection, SqliteRow);
