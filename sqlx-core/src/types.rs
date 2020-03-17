//! Traits linking Rust types to SQL types.

use std::fmt::{Debug, Display};

use crate::database::Database;

#[cfg(feature = "uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
pub use uuid::Uuid;

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub mod chrono {
    pub use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
}

pub trait TypeInfo: Debug + Display + Clone {
    /// Compares type information to determine if `other` is compatible at the Rust level
    /// with `self`.
    fn compatible(&self, other: &Self) -> bool;

    /// Return `true` if this is the database flavor's "null" sentinel type.
    ///
    /// For type info coming from the description of a prepared statement, this means
    /// that the server could not infer the expected type of a bind parameter or result column;
    /// the latter is most often the case with columns that are the result of an expression
    /// in a weakly-typed database like MySQL or SQLite.
    fn is_null_type(&self) -> bool;
}

/// Indicates that a SQL type is supported for a database.
pub trait Type<DB>
where
    DB: Database,
{
    /// Returns the canonical type information on the database for the type `T`.
    fn type_info() -> DB::TypeInfo;
}

// For references to types in Rust, the underlying SQL type information
// is equivalent
impl<T: ?Sized, DB> Type<DB> for &'_ T
where
    DB: Database,
    T: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }
}

// For optional types in Rust, the underlying SQL type information
// is equivalent
impl<T, DB> Type<DB> for Option<T>
where
    DB: Database,
    T: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }
}
