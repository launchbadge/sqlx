//! **MySQL** database and connection types.

use std::convert::TryInto;

pub use arguments::MySqlArguments;
pub use connection::MySqlConnection;
pub use database::MySql;
pub use row::MySqlRow;

use crate::url::Url;

mod arguments;
mod connection;
mod database;
mod error;
mod executor;
mod io;
mod protocol;
mod row;
mod types;

// used in tests and hidden code in examples
#[doc(hidden)]
pub async fn connect<T>(url: T) -> crate::Result<MySqlConnection>
    where
        T: TryInto<Url, Error = crate::Error>
{
    MySqlConnection::open(url.try_into()).await
}
