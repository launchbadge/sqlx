use std::fmt::Debug;
use std::hash::Hash;

use crate::{Column, QueryResult, RawValue, Row, TypeInfo};

/// A database driver.
///
/// Represents a family of traits for interacting with a database. This is
/// separate from [`Connection`][crate::Connection]. One database driver may
/// have multiple concrete `Connection` implementations.
///
pub trait Database:
    'static + Sized + Debug + for<'x> HasOutput<'x> + for<'r> HasRawValue<'r, Database = Self>
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
    type TypeId: 'static + PartialEq + Hash + Clone + Copy + Send + Sync;

    /// The character used to prefix bind parameter placeholders, e.g. `$` for Postgres, `?` for MySQL, etc.
    const PLACEHOLDER_CHAR: char;

    /// The indexing type for bind parameters.
    ///
    /// E.g. `Implicit` for MySQL which just does `SELECT 1 FROM foo WHERE bar = ? AND baz = ?`
    /// or `OneIndexed` for Postgres which does `SELECT 1 FROM foo WHERE bar = $1 AND baz = $2`
    const PARAM_INDEXING: crate::placeholders::ParamIndexing;
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
    type Database: Database;

    /// The concrete type to hold the input for [`Decode`] for this database.
    type RawValue: RawValue<'r, Database = Self::Database>;
}
