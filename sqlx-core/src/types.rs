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

#[cfg(feature = "bigdecimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigdecimal")))]
pub use bigdecimal::BigDecimal;

#[cfg(feature = "ipnetwork")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipnetwork")))]
pub mod ipnetwork {
    pub use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
}

pub trait TypeInfo: Debug + Display + Clone {
    /// Compares type information to determine if `other` is compatible at the Rust level
    /// with `self`.
    fn compatible(&self, other: &Self) -> bool;
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
