//! [MySQL] database driver.
//!
//! [MySQL]: https://www.mysql.com/
//!

use crate::DefaultRuntime;

pub type MySqlConnection<Rt = DefaultRuntime> = sqlx_mysql::MySqlConnection<Rt>;

pub use sqlx_mysql::{
    types, MySql, MySqlColumn, MySqlConnectOptions, MySqlDatabaseError, MySqlQueryResult,
    MySqlRawValue, MySqlRawValueFormat, MySqlRow, MySqlTypeId,
};
