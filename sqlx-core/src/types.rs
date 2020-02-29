//! Traits linking Rust types to SQL types.

use std::fmt::{Debug, Display};

use crate::Database;

#[cfg(feature = "uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
pub use uuid::Uuid;

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub mod chrono {
    pub use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
}

#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub mod time {
    pub use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};
}

pub trait TypeInfo: Debug + Display + Clone {
    /// Compares type information to determine if `other` is compatible at the Rust level
    /// with `self`.
    fn compatible(&self, other: &Self) -> bool;
}

/// Indicates that a SQL type is supported for a database.
pub trait HasSqlType<T: ?Sized>: Database {
    /// Returns the canonical type information on the database for the type `T`.
    fn type_info() -> Self::TypeInfo;
}

// For references to types in Rust, the underlying SQL type information
// is equivalent
impl<T: ?Sized, DB> HasSqlType<&'_ T> for DB
where
    DB: HasSqlType<T>,
{
    fn type_info() -> Self::TypeInfo {
        <DB as HasSqlType<T>>::type_info()
    }
}

// For optional types in Rust, the underlying SQL type information
// is equivalent
impl<T, DB> HasSqlType<Option<T>> for DB
where
    DB: HasSqlType<T>,
{
    fn type_info() -> Self::TypeInfo {
        <DB as HasSqlType<T>>::type_info()
    }
}
