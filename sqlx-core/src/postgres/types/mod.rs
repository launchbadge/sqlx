//! Conversions between Rust and **Postgres** types.
//!
//! # Types
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOL                                                 |
//! | `i16`                                 | SMALLINT, SMALLSERIAL, INT2                          |
//! | `i32`                                 | INT, SERIAL, INT4                                    |
//! | `i64`                                 | BIGINT, BIGSERIAL, INT8                              |
//! | `f32`                                 | REAL, FLOAT4                                         |
//! | `f64`                                 | DOUBLE PRECISION, FLOAT8                             |
//! | `&str`, `String`                      | VARCHAR, CHAR(N), TEXT, CITEXT, NAME                 |
//! | `&[u8]`, `Vec<u8>`                    | BYTEA                                                |
//!
//! ### [`chrono`](https://crates.io/crates/chrono)
//!
//! Requires the `chrono` Cargo feature flag.
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | `chrono::DateTime<Utc>`               | TIMESTAMPTZ                                          |
//! | `chrono::DateTime<Local>`             | TIMESTAMPTZ                                          |
//! | `chrono::NaiveDateTime`               | TIMESTAMP                                            |
//! | `chrono::NaiveDate`                   | DATE                                                 |
//! | `chrono::NaiveTime`                   | TIME                                                 |
//!
//! ### [`time`](https://crates.io/crates/time)
//!
//! Requires the `time` Cargo feature flag.
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | `time::PrimitiveDateTime`             | TIMESTAMP                                            |
//! | `time::OffsetDateTime`                | TIMESTAMPTZ                                          |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//!
//! ### [`uuid`](https://crates.io/crates/uuid)
//!
//! Requires the `uuid` Cargo feature flag.
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | `uuid::Uuid`                          | UUID                                                 |
//!
//! ### [`ipnetwork`](https://crates.io/crates/ipnetwork)
//!
//! Requires the `ipnetwork` Cargo feature flag.
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | `ipnetwork::IpNetwork`                | INET, CIDR                                           |
//!
//! ### [`json`](https://crates.io/crates/serde_json)
//!
//! Requires the `json` Cargo feature flag.
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | [`Json<T>`]                           | JSON, JSONB                                          |
//! | `serde_json::Value`                   | JSON, JSONB                                          |
//! | `&serde_json::value::RawValue`        | JSON, JSONB                                          |
//!
//! `Value` and `RawValue` from `serde_json` can be used for unstructured JSON data with
//! Postgres.
//!
//! [`Json<T>`] can be used for structured JSON data with Postgres.
//!
//! [`Json<T>`]: crate::types::Json
//!
//! # [Composite types](https://www.postgresql.org/docs/current/rowtypes.html)
//!
//! User-defined composite types are supported through a derive for `Type`.
//!
//! ```text
//! CREATE TYPE inventory_item AS (
//!     name            text,
//!     supplier_id     integer,
//!     price           numeric
//! );
//! ```
//!
//! ```rust,ignore
//! #[derive(sqlx::Type)]
//! #[sqlx(rename = "inventory_item")]
//! struct InventoryItem {
//!     name: String,
//!     supplier_id: i32,
//!     price: BigDecimal,
//! }
//! ```
//!
//! Anonymous composite types are represented as tuples. Note that anonymous composites may only
//! be returned and not sent to Postgres (this is a limitation of postgres).
//!
//! # Arrays
//!
//! One-dimensional arrays are supported as `Vec<T>` or `&[T]` where `T` implements `Type`.
//!
//! # [Enumerations](https://www.postgresql.org/docs/current/datatype-enum.html)
//!
//! User-defined enumerations are supported through a derive for `Type`.
//!
//! ```text
//! CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');
//! ```
//!
//! ```rust,ignore
//! #[derive(sqlx::Type)]
//! #[sqlx(rename = "mood", rename_all = "lowercase")]
//! enum Mood { Sad, Ok, Happy }
//! ```
//!
//! Rust enumerations may also be defined to be represented as an integer using `repr`.
//! The following type expects a SQL type of `INTEGER` or `INT4` and will convert to/from the
//! Rust enumeration.
//!
//! ```rust,ignore
//! #[derive(sqlx::Type)]
//! #[repr(i32)]
//! enum Mood { Sad = 0, Ok = 1, Happy = 2 }
//! ```
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from Postgres.
//!

use std::fmt::{self, Debug, Display};
use std::ops::Deref;
use std::sync::Arc;

use crate::decode::Decode;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgValue, Postgres};
use crate::types::TypeInfo;

mod array;
mod bool;
mod bytes;
mod float;
mod int;
mod record;
mod str;

// internal types used by other types to encode or decode related formats
#[doc(hidden)]
pub mod raw;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "time")]
mod time;

#[cfg(feature = "uuid")]
mod uuid;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "ipnetwork")]
mod ipnetwork;

/// Type information for a Postgres SQL type.
#[derive(Debug, Clone)]
pub struct PgTypeInfo {
    pub(crate) id: TypeId,
    pub(crate) name: Option<SharedStr>,
}

impl PgTypeInfo {
    pub(crate) fn new(id: TypeId, name: impl Into<SharedStr>) -> Self {
        Self {
            id,
            name: Some(name.into()),
        }
    }

    /// Create a `PgTypeInfo` from a type's object identifier.
    ///
    /// The object identifier of a type can be queried with
    /// `SELECT oid FROM pg_type WHERE typname = <name>;`
    pub fn with_oid(oid: u32) -> Self {
        Self {
            id: TypeId(oid),
            name: None,
        }
    }

    #[doc(hidden)]
    pub fn type_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<UNKNOWN>")
    }

    #[doc(hidden)]
    pub fn type_feature_gate(&self) -> Option<&'static str> {
        match self.id {
            TypeId::DATE | TypeId::TIME | TypeId::TIMESTAMP | TypeId::TIMESTAMPTZ => Some("chrono"),
            TypeId::UUID => Some("uuid"),
            // we can support decoding `PgNumeric` but it's decidedly less useful to the layman
            TypeId::NUMERIC => Some("bigdecimal"),
            TypeId::CIDR | TypeId::INET => Some("ipnetwork"),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn oid(&self) -> u32 {
        self.id.0
    }
}

impl Display for PgTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{}", *name)
        } else {
            write!(f, "{}", self.id)
        }
    }
}

impl PartialEq<PgTypeInfo> for PgTypeInfo {
    fn eq(&self, other: &PgTypeInfo) -> bool {
        // Postgres is strongly typed (mostly) so the rules that make sense here are equivalent
        // to the rules that make sense in [compatible]
        self.compatible(other)
    }
}

impl TypeInfo for PgTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        match (self.id, other.id) {
            (TypeId::CIDR, TypeId::INET)
            | (TypeId::INET, TypeId::CIDR)
            | (TypeId::ARRAY_CIDR, TypeId::ARRAY_INET)
            | (TypeId::ARRAY_INET, TypeId::ARRAY_CIDR) => true,

            // the following text-like types are compatible
            (TypeId::VARCHAR, other)
            | (TypeId::TEXT, other)
            | (TypeId::BPCHAR, other)
            | (TypeId::NAME, other)
            | (TypeId::UNKNOWN, other)
                if match other {
                    TypeId::VARCHAR
                    | TypeId::TEXT
                    | TypeId::BPCHAR
                    | TypeId::NAME
                    | TypeId::UNKNOWN => true,
                    _ => false,
                } =>
            {
                true
            }

            // the following text-like array types are compatible
            (TypeId::ARRAY_VARCHAR, other)
            | (TypeId::ARRAY_TEXT, other)
            | (TypeId::ARRAY_BPCHAR, other)
            | (TypeId::ARRAY_NAME, other)
                if match other {
                    TypeId::ARRAY_VARCHAR
                    | TypeId::ARRAY_TEXT
                    | TypeId::ARRAY_BPCHAR
                    | TypeId::ARRAY_NAME => true,
                    _ => false,
                } =>
            {
                true
            }

            // JSON <=> JSONB
            (TypeId::JSON, other) | (TypeId::JSONB, other)
                if match other {
                    TypeId::JSON | TypeId::JSONB => true,
                    _ => false,
                } =>
            {
                true
            }

            _ => self.id.0 == other.id.0,
        }
    }
}

impl<'de, T> Decode<'de, Postgres> for Option<T>
where
    T: Decode<'de, Postgres>,
{
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        Ok(if value.get().is_some() {
            Some(<T as Decode<Postgres>>::decode(value)?)
        } else {
            None
        })
    }
}

/// Copy of `Cow` but for strings; clones guaranteed to be cheap.
#[derive(Clone, Debug)]
pub(crate) enum SharedStr {
    Static(&'static str),
    Arc(Arc<str>),
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            SharedStr::Static(s) => s,
            SharedStr::Arc(s) => s,
        }
    }
}

impl<'a> From<&'a SharedStr> for SharedStr {
    fn from(s: &'a SharedStr) -> Self {
        s.clone()
    }
}

impl From<&'static str> for SharedStr {
    fn from(s: &'static str) -> Self {
        SharedStr::Static(s)
    }
}

impl From<String> for SharedStr {
    #[inline]
    fn from(s: String) -> Self {
        SharedStr::Arc(s.into())
    }
}

impl fmt::Display for SharedStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self)
    }
}
