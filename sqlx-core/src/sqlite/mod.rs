mod arguments;
mod connection;
mod cursor;
mod database;
mod error;
mod executor;
mod row;
mod types;
mod value;

pub use arguments::SqliteArguments;
pub use connection::SqliteConnection;
pub use cursor::SqliteCursor;
pub use database::Sqlite;
pub use error::SqliteError;
pub use row::SqliteRow;
pub use types::SqliteTypeInfo;

/// An alias for [`Pool`][crate::Pool], specialized for **Sqlite**.
pub type SqlitePool = crate::pool::Pool<SqliteConnection>;

make_query_as!(SqliteQueryAs, Sqlite, SqliteRow);
impl_map_row_for_row!(Sqlite, SqliteRow);
impl_column_index_for_row!(Sqlite);
impl_from_row_for_tuples!(Sqlite, SqliteRow);
