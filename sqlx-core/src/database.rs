use std::fmt::{Debug, Display};

use crate::arguments::Arguments;
use crate::connection::Connect;
use crate::row::Row;
use crate::type_info::TypeInfo;
use crate::value::RawValue;

/// A database driver.
///
/// This trait encapsulates a complete set of traits that implement a driver for a
/// specific database (e.g., MySQL, PostgreSQL).
pub trait Database:
    Sized
    + Send
    + Debug
    + for<'r> HasRawValue<'r, Database = Self>
    + for<'q> HasArguments<'q, Database = Self>
{
    /// The concrete `Connection` implementation for this database.
    type Connection: Connect<Database = Self>;

    /// The concrete `Row` implementation for this database.
    type Row: Row<Database = Self>;

    /// The concrete `TypeInfo` implementation for this database.
    type TypeInfo: TypeInfo;

    /// The concrete type used to collect values when binding arguments for a prepared query.
    type RawBuffer: Default;
}

/// Associate [`Database`] with a [`RawValue`] of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasRawValue<'r> {
    type Database: Database;

    /// The concrete type used to hold a not-yet-decoded value that has just been
    /// received from the database.
    type RawValue: RawValue<'r, Database = Self::Database>;
}

/// Associate [`Database`] with an [`Arguments`] of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasArguments<'q> {
    type Database: Database;

    /// The concrete `Arguments` implementation for this database.
    type Arguments: Arguments<'q, Database = Self::Database>;
}
