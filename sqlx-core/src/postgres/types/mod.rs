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
//! | `&str`, `String`                      | VARCHAR, CHAR(N), TEXT, NAME                         |
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
//! ### [`geo`](https://crates.io/crates/geo)
//!
//! Requires the `geo` Cargo feature flag.
//!
//! | Rust type                             | Postgres type(s)                                     |
//! |---------------------------------------|------------------------------------------------------|
//! | `geo::Coordinate<f64>`                | POINT                                                |
//! | `geo::Line<f64>`                      | LINE, LSEG                                           |
//! | `geo::Rect<f64>`                      | BOX                                                  |
//! | `geo::LineString<f64>`                | PATH                                                 |
//! | `geo::Polygon<f64>`                   | POLYGON                                              |
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

use crate::decode::Decode;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgValue, Postgres};

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

#[cfg(feature = "geo")]
mod geo;

// Implement `Decode` for all postgres types
// The concept of a nullable `RawValue` is db-specific
// `Type` is implemented generically at src/types.rs
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

// Try to resolve a _static_ type name from an OID
pub(crate) fn try_resolve_type_name(oid: u32) -> Option<&'static str> {
    Some(match TypeId(oid) {
        TypeId::BOOL => "BOOL",

        TypeId::CHAR => "\"CHAR\"",

        TypeId::INT2 => "INT2",
        TypeId::INT4 => "INT4",
        TypeId::INT8 => "INT8",

        TypeId::OID => "OID",

        TypeId::FLOAT4 => "FLOAT4",
        TypeId::FLOAT8 => "FLOAT8",

        TypeId::NUMERIC => "NUMERIC",

        TypeId::TEXT => "TEXT",
        TypeId::VARCHAR => "VARCHAR",
        TypeId::BPCHAR => "BPCHAR",
        TypeId::UNKNOWN => "UNKNOWN",
        TypeId::NAME => "NAME",

        TypeId::DATE => "DATE",
        TypeId::TIME => "TIME",
        TypeId::TIMESTAMP => "TIMESTAMP",
        TypeId::TIMESTAMPTZ => "TIMESTAMPTZ",

        TypeId::BYTEA => "BYTEA",

        TypeId::UUID => "UUID",

        TypeId::CIDR => "CIDR",
        TypeId::INET => "INET",

        TypeId::ARRAY_BOOL => "BOOL[]",

        TypeId::ARRAY_CHAR => "\"CHAR\"[]",

        TypeId::ARRAY_INT2 => "INT2[]",
        TypeId::ARRAY_INT4 => "INT4[]",
        TypeId::ARRAY_INT8 => "INT8[]",

        TypeId::ARRAY_OID => "OID[]",

        TypeId::ARRAY_FLOAT4 => "FLOAT4[]",
        TypeId::ARRAY_FLOAT8 => "FLOAT8[]",

        TypeId::ARRAY_TEXT => "TEXT[]",
        TypeId::ARRAY_VARCHAR => "VARCHAR[]",
        TypeId::ARRAY_BPCHAR => "BPCHAR[]",
        TypeId::ARRAY_NAME => "NAME[]",

        TypeId::ARRAY_NUMERIC => "NUMERIC[]",

        TypeId::ARRAY_DATE => "DATE[]",
        TypeId::ARRAY_TIME => "TIME[]",
        TypeId::ARRAY_TIMESTAMP => "TIMESTAMP[]",
        TypeId::ARRAY_TIMESTAMPTZ => "TIMESTAMPTZ[]",

        TypeId::ARRAY_BYTEA => "BYTEA[]",

        TypeId::ARRAY_UUID => "UUID[]",

        TypeId::ARRAY_CIDR => "CIDR[]",
        TypeId::ARRAY_INET => "INET[]",

        TypeId::JSON => "JSON",
        TypeId::JSONB => "JSONB",

        TypeId::RECORD => "RECORD",
        TypeId::ARRAY_RECORD => "RECORD[]",

        _ => {
            return None;
        }
    })
}
