//! **SQLite** database driver.

// SQLite is a C library. All interactions require FFI which is unsafe.
// All unsafe blocks should have comments pointing to SQLite docs and ensuring that we maintain
// invariants.
#![allow(unsafe_code)]

pub use arguments::{SqliteArgumentValue, SqliteArguments};
pub use column::SqliteColumn;
pub use connection::{LockedSqliteHandle, SqliteConnection};
pub use database::Sqlite;
pub use error::SqliteError;
pub use options::{
    SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode, SqliteSynchronous,
};
pub use query_result::SqliteQueryResult;
pub use row::SqliteRow;
pub use statement::SqliteStatement;
use std::sync::atomic::AtomicBool;
pub use transaction::SqliteTransactionManager;
pub use type_info::SqliteTypeInfo;
pub use value::{SqliteValue, SqliteValueRef};

use crate::describe::Describe;
use crate::error::Error;
use crate::executor::Executor;
use crate::sqlite::connection::establish::EstablishParams;

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod options;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

#[cfg(feature = "migrate")]
mod migrate;

#[cfg(feature = "migrate")]
mod testing;

/// An alias for [`Pool`][crate::pool::Pool], specialized for SQLite.
pub type SqlitePool = crate::pool::Pool<Sqlite>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for SQLite.
pub type SqlitePoolOptions = crate::pool::PoolOptions<Sqlite>;

/// An alias for [`Executor<'_, Database = Sqlite>`][Executor].
pub trait SqliteExecutor<'c>: Executor<'c, Database = Sqlite> {}
impl<'c, T: Executor<'c, Database = Sqlite>> SqliteExecutor<'c> for T {}

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(SqliteArguments<'q>);
impl_executor_for_pool_connection!(Sqlite, SqliteConnection, SqliteRow);
impl_executor_for_transaction!(Sqlite, SqliteRow);
impl_column_index_for_row!(SqliteRow);
impl_column_index_for_statement!(SqliteStatement);
impl_acquire!(Sqlite, SqliteConnection);
impl_into_maybe_pool!(Sqlite, SqliteConnection);

// required because some databases have a different handling of NULL
impl_encode_for_option!(Sqlite);

/// UNSTABLE: for use by `sqlx-cli` only.
#[doc(hidden)]
pub static CREATE_DB_WAL: AtomicBool = AtomicBool::new(true);

/// UNSTABLE: for use by `sqlite_macros` only.
#[doc(hidden)]
pub fn describe_blocking(
    opts: &SqliteConnectOptions,
    query: &str,
) -> Result<Describe<Sqlite>, Error> {
    let params = EstablishParams::from_options(opts)?;
    let mut conn = params.establish()?;

    // Execute any ancillary `PRAGMA`s
    connection::execute::iter(&mut conn, &opts.pragma_string(), None, false)?.finish()?;

    connection::describe::describe(&mut conn, query)

    // SQLite database is closed immediately when `conn` is dropped
}
