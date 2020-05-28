//! **MySQL** database driver.

mod arguments;
mod connection;
mod database;
mod error;
mod io;
mod options;
mod protocol;
mod row;
mod type_info;
pub mod types;
mod value;

pub use arguments::MySqlArguments;
pub use connection::MySqlConnection;
pub use database::MySql;
pub use error::MySqlDatabaseError;
pub use options::{MySqlConnectOptions, MySqlSslMode};
pub use row::MySqlRow;
pub use type_info::MySqlTypeInfo;
pub use value::{MySqlValue, MySqlValueFormat, MySqlValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for MySQL.
pub type MySqlPool = crate::pool::Pool<MySqlConnection>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(MySqlArguments);
impl_executor_for_pool_connection!(MySql, MySqlConnection, MySqlRow);
