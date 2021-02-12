//! [MySQL] database driver.
//!
//! [MySQL]: https://www.mysql.com/
//!

mod connection;
mod options;

#[cfg(feature = "blocking")]
mod blocking;

//
// these types are wrapped instead of re-exported
// this is to provide runtime-specialized inherent methods by taking advantage
// of through crate-local negative reasoning
pub use connection::MySqlConnection;
pub use options::MySqlConnectOptions;
//
// re-export the remaining types from the driver
pub use sqlx_mysql::{
    types, MySql, MySqlColumn, MySqlDatabaseError, MySqlQueryResult, MySqlRawValue,
    MySqlRawValueFormat, MySqlRow, MySqlTypeId,
};
