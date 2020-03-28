//! **MySQL** database and connection types.

pub use arguments::MySqlArguments;
pub use connection::MySqlConnection;
pub use cursor::MySqlCursor;
pub use database::MySql;
pub use error::MySqlError;
pub use row::MySqlRow;
pub use types::MySqlTypeInfo;
pub use value::{MySqlData, MySqlValue};

mod arguments;
mod connection;
mod cursor;
mod database;
mod error;
mod executor;
mod io;
mod protocol;
mod row;
mod rsa;
mod stream;
mod tls;
pub mod types;
mod util;
mod value;

/// An alias for [`crate::pool::Pool`], specialized for **MySQL**.
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub type MySqlPool = crate::pool::Pool<MySqlConnection>;

make_query_as!(MySqlQueryAs, MySql, MySqlRow);
impl_map_row_for_row!(MySql, MySqlRow);
impl_from_row_for_tuples!(MySql, MySqlRow);
