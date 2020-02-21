use std::fmt::Display;

use crate::arguments::Arguments;
use crate::connection::Connection;
use crate::cursor::Cursor;
use crate::row::Row;
use crate::types::TypeInfo;

/// A database driver.
///
/// This trait encapsulates a complete driver implementation to a specific
/// database (e.g., MySQL, Postgres).
pub trait Database
where
    Self: Sized + 'static,
    Self: for<'a> HasRow<'a, Database = Self>,
    Self: for<'a> HasRawValue<'a>,
    Self: for<'c, 'q> HasCursor<'c, 'q, Self>,
{
    /// The concrete `Connection` implementation for this database.
    type Connection: Connection<Database = Self>;

    /// The concrete `Arguments` implementation for this database.
    type Arguments: Arguments<Database = Self>;

    /// The concrete `TypeInfo` implementation for this database.
    type TypeInfo: TypeInfo;

    /// The Rust type of table identifiers for this database.
    type TableId: Display + Clone;
}

pub trait HasRawValue<'a> {
    type RawValue;
}

pub trait HasCursor<'c, 'q, DB>
where
    DB: Database,
{
    type Cursor: Cursor<'c, 'q, DB>;
}

pub trait HasRow<'a> {
    type Database: Database;

    type Row: Row<'a, Database = Self::Database>;
}
