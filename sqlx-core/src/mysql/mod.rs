//! **MySQL** database driver.

mod arguments;
mod connection;
mod database;
mod error;
mod io;
mod options;
mod protocol;
mod row;
mod transaction;
mod type_info;
pub mod types;
mod value;

pub use arguments::MySqlArguments;
pub use connection::MySqlConnection;
pub use database::MySql;
pub use error::MySqlDatabaseError;
pub use options::{MySqlConnectOptions, MySqlSslMode};
pub use row::MySqlRow;
pub use transaction::MySqlTransactionManager;
pub use type_info::MySqlTypeInfo;
pub use value::{MySqlValue, MySqlValueFormat, MySqlValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for MySQL.
pub type MySqlPool = crate::pool::Pool<MySql>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(MySqlArguments);
impl_executor_for_pool_connection!(MySql, MySqlConnection, MySqlRow);
impl_executor_for_transaction!(MySql, MySqlRow);
impl_map_row!(MySql, MySqlRow);

// required because some databases have a different handling of NULL
impl_encode_for_option!(MySql);
