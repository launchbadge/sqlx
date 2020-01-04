//! **MySQL** database and connection types.

mod arguments;
mod connection;
mod database;
mod error;
mod executor;
mod io;
mod protocol;
mod row;
mod rsa;
mod types;
mod util;

pub use database::MySql;

pub use arguments::MySqlArguments;

pub use connection::MySqlConnection;

pub use error::MySqlError;

pub use types::MySqlTypeInfo;

pub use row::MySqlRow;

/// An alias for [`Pool`], specialized for **MySQL**.
pub type MySqlPool = super::Pool<MySql>;
