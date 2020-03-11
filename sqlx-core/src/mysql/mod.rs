//! **MySQL** database and connection types.

pub use arguments::MySqlArguments;
pub use connection::MySqlConnection;
pub use cursor::MySqlCursor;
pub use database::MySql;
pub use error::MySqlError;
pub use row::{MySqlRow, MySqlValue};
pub use types::MySqlTypeInfo;

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
mod types;
mod util;

/// An alias for [`Pool`], specialized for **MySQL**.
pub type MySqlPool = crate::pool::Pool<MySqlConnection>;

make_query_as!(MySqlQueryAs, MySql, MySqlRow);
impl_map_row_for_row!(MySql, MySqlRow);
impl_column_index_for_row!(MySql);
impl_from_row_for_tuples!(MySql, MySqlRow);
