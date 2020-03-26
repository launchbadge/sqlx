//! **SQLite** database and connection types.

mod arguments;
mod connection;
mod cursor;
mod database;
mod error;
mod executor;
mod row;
mod statement;
pub mod types;
mod value;
mod worker;

pub use arguments::{SqliteArgumentValue, SqliteArguments};
pub use connection::SqliteConnection;
pub use cursor::SqliteCursor;
pub use database::Sqlite;
pub use error::SqliteError;
pub use row::SqliteRow;
pub use types::SqliteTypeInfo;
pub use value::SqliteValue;

/// An alias for [`Pool`][crate::pool::Pool], specialized for **Sqlite**.
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub type SqlitePool = crate::pool::Pool<SqliteConnection>;

make_query_as!(SqliteQueryAs, Sqlite, SqliteRow);
impl_map_row_for_row!(Sqlite, SqliteRow);
impl_from_row_for_tuples!(Sqlite, SqliteRow);
