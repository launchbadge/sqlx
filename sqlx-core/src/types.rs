//! Conversions between Rust and SQL types.
//!
//! To see how each SQL type maps to a Rust type, see the corresponding `types` module for each
//! database:
//!
//!  * [PostgreSQL](../postgres/types/index.html)
//!  * [MySQL](../mysql/types/index.html)
//!  * [SQLite](../sqlite/types/index.html)
//!
//! Any external types that have had [`Type`] implemented for, are re-exported in this module
//! for convenience as downstream users need to use a compatible version of the external crate
//! to take advantage of the implementation.

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

#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub mod time {
    pub use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};
}

#[cfg(feature = "bigdecimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigdecimal")))]
pub use bigdecimal::BigDecimal;

#[cfg(feature = "ipnetwork")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipnetwork")))]
pub mod ipnetwork {
    pub use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
}

#[cfg(feature = "json")]
pub mod json {
    use std::ops::Deref;

    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct Json<T>(pub T);

    impl<T> Deref for Json<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> AsRef<T> for Json<T> {
        fn as_ref(&self) -> &T {
            &self.0
        }
    }
}
#[cfg(feature = "json")]
pub use self::json::Json;

pub trait TypeInfo: PartialEq<Self> + Debug + Display + Clone {
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
