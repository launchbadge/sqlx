use std::fmt::{Debug, Display};

use crate::arguments::Arguments;
use crate::connection::Connect;
use crate::row::Row;
use crate::type_info::TypeInfo;
use crate::value::{Value, ValueRef};

/// A database driver.
///
/// This trait encapsulates a complete set of traits that implement a driver for a
/// specific database (e.g., MySQL, PostgreSQL).
pub trait Database:
    Sized
    + Send
    + Debug
    + for<'r> HasValueRef<'r, Database = Self>
    + for<'q> HasArguments<'q, Database = Self>
{
    /// The concrete `Connection` implementation for this database.
    type Connection: Connect<Database = Self>;

    /// The concrete `Row` implementation for this database.
    type Row: Row<Database = Self>;

    /// The concrete `TypeInfo` implementation for this database.
    type TypeInfo: TypeInfo;

    /// The concrete type used to hold a owned copy of the not-yet-decoded value that was
    /// received from the database.
    type Value: Value<Database = Self> + 'static;
}

/// Associate [`Database`] with a [`ValueRef`](crate::value::ValueRef) of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasValueRef<'r> {
    type Database: Database;

    /// The concrete type used to hold a reference to the not-yet-decoded value that has just been
    /// received from the database.
    type ValueRef: ValueRef<'r, Database = Self::Database>;
}

/// Associate [`Database`] with an [`Arguments`](crate::arguments::Arguments) of a generic lifetime.
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
