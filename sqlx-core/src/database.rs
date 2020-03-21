use std::fmt::{Debug, Display};

use crate::arguments::Arguments;
use crate::connection::Connect;
use crate::cursor::Cursor;
use crate::error::DatabaseError;
use crate::row::Row;
use crate::types::TypeInfo;

/// A database driver.
///
/// This trait encapsulates a complete driver implementation to a specific
/// database (e.g., **MySQL**, **Postgres**).
pub trait Database
where
    Self: Sized + Send + 'static,
    Self: for<'c> HasRow<'c, Database = Self>,
    Self: for<'c> HasRawValue<'c>,
    Self: for<'c, 'q> HasCursor<'c, 'q, Database = Self>,
    Self: Debug,
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
    /// For example, **Postgres** and **MySQL** use `Vec<u8>`; however, **SQLite** uses `Vec<SqliteArgumentValue>`.
    type RawBuffer: Default;

    /// The concrete `DatabaseError` type used to report errors from the database.
    type Error: DatabaseError + Send + Sync;
}

/// Associate [`Database`] with a `RawValue` of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasRawValue<'c> {
    /// The Rust type used to hold a not-yet-decoded value that has just been
    /// received from the database.
    ///
    /// For example, **Postgres** and **MySQL** use `&'c [u8]`; however, **SQLite** uses `SqliteValue<'c>`.
    type RawValue;
}

/// Associate [`Database`] with a [`Cursor`] of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasCursor<'c, 'q> {
    type Database: Database;

    /// The concrete `Cursor` implementation for this database.
    type Cursor: Cursor<'c, 'q, Database = Self::Database>;
}

/// Associate [`Database`] with a [`Row`] of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasRow<'c> {
    type Database: Database;

    /// The concrete `Row` implementation for this database.
    type Row: Row<'c, Database = Self::Database>;
}
