use std::fmt::Debug;

use crate::{Column, Connection, QueryResult, Row, Runtime};

/// A database driver.
///
/// This trait encapsulates a complete set of traits that implement a driver for a
/// specific database (e.g., MySQL, PostgreSQL).
///
pub trait Database<Rt>: 'static + Sized + Debug + for<'x> HasOutput<'x>
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
}

/// Associates [`Database`] with a `Output` of a generic lifetime.
/// 'x: single execution
pub trait HasOutput<'x> {
    /// The concrete type to hold the output for `Encode` for this database. This may be
    /// a simple alias to `&'x mut Vec<u8>`.
    type Output;
}
