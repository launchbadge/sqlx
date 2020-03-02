use std::fmt::Display;

use crate::arguments::Arguments;
use crate::connection::{Connect, Connection};
use crate::cursor::Cursor;
use crate::query_as::QueryAs;
use crate::row::Row;
use crate::types::TypeInfo;

/// A database driver.
///
/// This trait encapsulates a complete driver implementation to a specific
/// database (e.g., MySQL, Postgres).
pub trait Database
where
    Self: Sized + Send + 'static,
    Self: for<'c> HasRow<'c, Database = Self>,
    Self: for<'c> HasRawValue<'c>,
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
}

pub trait HasRawValue<'c> {
    type RawValue;
}

pub trait HasCursor<'c, 'q> {
    type Database: Database;

    type Cursor: Cursor<'c, 'q, Database = Self::Database>;
}

pub trait HasRow<'c> {
    type Database: Database;

    type Row: Row<'c, Database = Self::Database>;
}
