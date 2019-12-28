//! Traits linking Rust types to SQL types.

use std::fmt::Display;

#[cfg(feature = "uuid")]
pub use uuid::Uuid;

#[cfg(feature = "chrono")]
pub mod chrono {
    pub use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
}

/// Information about how a database stores metadata about given SQL types.
pub trait HasTypeMetadata {
    /// The actual type used to represent metadata.
    type TypeMetadata: PartialEq<Self::TypeId>;

    /// The Rust type of table identifiers.
    type TableId: Display;

    /// The Rust type of type identifiers.
    type TypeId: Display;
}

/// Indicates that a SQL type is supported for a database.
pub trait HasSqlType<T: ?Sized>: HasTypeMetadata {
    /// Fetch the metadata for the given type.
    fn metadata() -> Self::TypeMetadata;
}

impl<T: ?Sized, DB> HasSqlType<&'_ T> for DB
where
    DB: HasSqlType<T>,
{
    fn metadata() -> Self::TypeMetadata {
        <DB as HasSqlType<T>>::metadata()
    }
}

impl<T, DB> HasSqlType<Option<T>> for DB
where
    DB: HasSqlType<T>,
{
    fn metadata() -> Self::TypeMetadata {
        <DB as HasSqlType<T>>::metadata()
    }
}
