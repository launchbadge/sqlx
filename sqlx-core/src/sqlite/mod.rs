mod arguments;
mod connection;
mod cursor;
mod database;
mod error;
mod executor;
mod row;
mod statement;
mod types;
mod value;

pub use arguments::{SqliteArgumentValue, SqliteArguments};
pub use connection::SqliteConnection;
pub use cursor::SqliteCursor;
pub use database::Sqlite;
pub use error::SqliteError;
pub use row::SqliteRow;
pub use types::SqliteTypeInfo;
pub use value::SqliteResultValue;

/// An alias for [`Pool`][crate::Pool], specialized for **Sqlite**.
pub type SqlitePool = crate::pool::Pool<SqliteConnection>;

make_query_as!(SqliteQueryAs, Sqlite, SqliteRow);
impl_map_row_for_row!(Sqlite, SqliteRow);
impl_from_row_for_tuples!(Sqlite, SqliteRow);
