use std::fmt::{Debug, Display};

use crate::arguments::Arguments;
use crate::connection::Connect;
use crate::cursor::HasCursor;
use crate::error::DatabaseError;
use crate::row::HasRow;
use crate::types::TypeInfo;
use crate::value::HasRawValue;

/// A database driver.
///
/// This trait encapsulates a complete driver implementation to a specific
/// database (e.g., **MySQL**, **Postgres**).
pub trait Database
where
    Self: Debug + Sized + Send + 'static,
    Self: for<'c> HasRow<'c, Database = Self>,
    Self: for<'c> HasRawValue<'c, Database = Self>,
    Self: for<'c, 'q> HasCursor<'c, 'q, Database = Self>,
{
    /// The concrete `Connection` implementation for this database.
    type Connection: Connect<Database = Self>;

    /// The concrete `Arguments` implementation for this database.
    type Arguments: Arguments<Database = Self>;

    /// The concrete `TypeInfo` implementation for this database.
    type TypeInfo: TypeInfo;

    /// The Rust type of table identifiers for this database.
    type TableId: Display + Clone;

    /// The Rust type used as the buffer when encoding arguments.
    ///
    /// For example, **Postgres** and **MySQL** use `Vec<u8>`;
    /// however, **SQLite** uses `Vec<SqliteArgumentValue>`.
    type RawBuffer: Default;

    /// The concrete `DatabaseError` type used to report errors from the database.
    type Error: DatabaseError + Send + Sync;
}
