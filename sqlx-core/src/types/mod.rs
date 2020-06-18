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

use crate::database::Database;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
mod json;

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
pub use json::Json;

/// Indicates that a SQL type is supported for a database.
pub trait Type<DB: Database> {
    /// Returns the canonical SQL type for this Rust type.
    ///
    /// When binding arguments, this is used to tell the database what is about to be sent; which,
    /// the database then uses to guide query plans. This can be overridden by `Encode::produces`.
    ///
    /// A map of SQL types to Rust types is populated with this and used
    /// to determine the type that is returned from the anonymous struct type from `query!`.
    fn type_info() -> DB::TypeInfo;

    /// Determines if this Rust type is compatible with the given SQL type.
    ///
    /// When decoding values from a row, this method is checked to determine if we should continue
    /// or raise a runtime type mismatch error.
    ///
    /// When binding arguments with `query!` or `query_as!`, this method is consulted to determine
    /// if the Rust type is acceptable.
    fn compatible(ty: &DB::TypeInfo) -> bool {
        *ty == Self::type_info()
    }
}

// for references, the underlying SQL type is identical
impl<T: ?Sized + Type<DB>, DB: Database> Type<DB> for &'_ T {
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <T as Type<DB>>::compatible(ty)
    }
}

// for optionals, the underlying SQL type is identical
impl<T: Type<DB>, DB: Database> Type<DB> for Option<T> {
    fn type_info() -> DB::TypeInfo {
        <T as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <T as Type<DB>>::compatible(ty)
    }
}
