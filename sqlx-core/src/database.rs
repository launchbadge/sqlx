use std::fmt::Debug;
use std::hash::Hash;

use crate::{Column, QueryResult, Row, TypeInfo};

/// A database driver.
///
/// Represents a family of traits for interacting with a database. This is
/// separate from [`Connection`][crate::Connection]. One database driver may
/// have multiple concrete `Connection` implementations.
///
pub trait Database:
    'static + Sized + Debug + for<'x> HasOutput<'x> + for<'r> HasRawValue<'r>
{
    /// The concrete [`Column`] implementation for this database.
    type Column: Column<Database = Self>;

    /// The concrete [`Row`] implementation for this database.
    type Row: Row<Database = Self>;

    /// The concrete [`QueryResult`] implementation for this database.
    type QueryResult: QueryResult;

    /// The concrete [`TypeInfo`] implementation for this database.
    type TypeInfo: TypeInfo<Database = Self>;

    /// The concrete [`TypeId`] implementation for this database.
    type TypeId: PartialEq + Hash + Clone + Copy;
}

/// Associates [`Database`] with an `Output` of a generic lifetime.
// 'x: single execution
pub trait HasOutput<'x> {
    /// The concrete type to hold the output for [`Encode`] for this database.
    type Output;
}

/// Associates [`Database`] with a `RawValue` of a generic lifetime.
// 'r: row
pub trait HasRawValue<'r> {
    /// The concrete type to hold the input for [`Decode`] for this database.
    type RawValue;
}
