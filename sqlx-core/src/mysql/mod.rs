//! **MySQL** database and connection types.

mod arguments;
mod connection;
mod database;
mod error;
mod executor;
mod io;
mod protocol;
mod row;
mod types;

pub use database::MySql;

pub use arguments::MySqlArguments;

pub use connection::MySqlConnection;

pub use error::MySqlError;

pub use row::MySqlRow;

/// An alias for [`Pool`], specialized for **MySQL**.
pub type MySqlPool = super::Pool<MySql>;

use std::convert::TryInto;

use crate::url::Url;

// used in tests and hidden code in examples
#[doc(hidden)]
pub async fn connect<T>(url: T) -> crate::Result<MySqlConnection>
    where
        T: TryInto<Url, Error = crate::Error>
{
    MySqlConnection::open(url.try_into()).await
}
