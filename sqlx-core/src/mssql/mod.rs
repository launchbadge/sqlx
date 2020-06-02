//! Microsoft SQL (MSSQL) database driver.

mod arguments;
mod connection;
mod database;
mod error;
mod options;
mod row;
mod transaction;
mod type_info;
mod value;

pub use arguments::MsSqlArguments;
pub use connection::MsSqlConnection;
pub use database::MsSql;
pub use error::MsSqlDatabaseError;
pub use options::MsSqlConnectOptions;
pub use row::MsSqlRow;
pub use transaction::MsSqlTransactionManager;
pub use type_info::MsSqlTypeInfo;
pub use value::{MsSqlValue, MsSqlValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for MySQL.
pub type MsSqlPool = crate::pool::Pool<MsSql>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(MsSqlArguments);
impl_executor_for_pool_connection!(MsSql, MsSqlConnection, MsSqlRow);
impl_executor_for_transaction!(MsSql, MsSqlRow);
