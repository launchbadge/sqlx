use std::fmt::Debug;

use crate::{Column, Connection, QueryResult, Row, Runtime};

/// A database driver.
///
/// This trait encapsulates a complete set of traits that implement a driver for a
/// specific database (e.g., MySQL, PostgreSQL).
///
pub trait Database<Rt>:
    'static + Sized + Debug + for<'x> HasOutput<'x> + for<'r> HasRawValue<'r>
where
    Rt: Runtime,
{
    /// The concrete [`Connection`] implementation for this database.
    type Connection: Connection<Rt, Database = Self> + ?Sized;

    /// The concrete [`Column`] implementation for this database.
    type Column: Column;

    /// The concrete [`Row`] implementation for this database.
    type Row: Row<Column = Self::Column>;

    /// The concrete [`QueryResult`] implementation for this database.
    type QueryResult: QueryResult;

    /// The concrete [`TypeInfo`] implementation for this database.
    type TypeInfo;

    /// The concrete [`TypeId`] implementation for this database.
    type TypeId;
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
