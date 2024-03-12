//! Conversions between Rust and **MySQL/MariaDB** types.
//!
//! # Types
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | TINYINT(1), BOOLEAN, BOOL (see below)                |
//! | `i8`                                  | TINYINT                                              |
//! | `i16`                                 | SMALLINT                                             |
//! | `i32`                                 | INT                                                  |
//! | `i64`                                 | BIGINT                                               |
//! | `u8`                                  | TINYINT UNSIGNED                                     |
//! | `u16`                                 | SMALLINT UNSIGNED                                    |
//! | `u32`                                 | INT UNSIGNED                                         |
//! | `u64`                                 | BIGINT UNSIGNED                                      |
//! | `f32`                                 | FLOAT                                                |
//! | `f64`                                 | DOUBLE                                               |
//! | `&str`, [`String`]                    | VARCHAR, CHAR, TEXT                                  |
//! | `&[u8]`, `Vec<u8>`                    | VARBINARY, BINARY, BLOB                              |
//! | `IpAddr`                              | VARCHAR, TEXT                                        |
//! | `Ipv4Addr`                            | INET4 (MariaDB-only), VARCHAR, TEXT                  |
//! | `Ipv6Addr`                            | INET6 (MariaDB-only), VARCHAR, TEXT                  |
//!
//! ##### Note: `BOOLEAN`/`BOOL` Type
//! MySQL and MariaDB treat `BOOLEAN` as an alias of the `TINYINT` type:
//!
//! * [Using Data Types from Other Database Engines (MySQL)](https://dev.mysql.com/doc/refman/8.0/en/other-vendor-data-types.html)
//! * [BOOLEAN (MariaDB)](https://mariadb.com/kb/en/boolean/)
//!
//! For the most part, you can simply use the Rust type `bool` when encoding or decoding a value
//! using the dynamic query interface, or passing a boolean as a parameter to the query macros
//! (`query!()` _et al._).
//!
//! However, because the MySQL wire protocol does not distinguish between `TINYINT` and `BOOLEAN`,
//! the query macros cannot know that a `TINYINT` column is semantically a boolean.
//! By default, they will map a `TINYINT` column as `i8` instead, as that is the safer assumption.
//!
//! Thus, you must use the type override syntax in the query to tell the macros you are expecting
//! a `bool` column. See the docs for `query!()` and `query_as!()` for details on this syntax.
//!
//! ### [`chrono`](https://crates.io/crates/chrono)
//!
//! Requires the `chrono` Cargo feature flag.
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | `chrono::DateTime<Utc>`               | TIMESTAMP                                            |
//! | `chrono::DateTime<Local>`             | TIMESTAMP                                            |
//! | `chrono::NaiveDateTime`               | DATETIME                                             |
//! | `chrono::NaiveDate`                   | DATE                                                 |
//! | `chrono::NaiveTime`                   | TIME                                                 |
//!
//! ### [`time`](https://crates.io/crates/time)
//!
//! Requires the `time` Cargo feature flag.
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | `time::PrimitiveDateTime`             | DATETIME                                             |
//! | `time::OffsetDateTime`                | TIMESTAMP                                            |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//!
//! ### [`bigdecimal`](https://crates.io/crates/bigdecimal)
//! Requires the `bigdecimal` Cargo feature flag.
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | `bigdecimal::BigDecimal`              | DECIMAL                                              |
//!
//! ### [`decimal`](https://crates.io/crates/rust_decimal)
//! Requires the `decimal` Cargo feature flag.
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | `rust_decimal::Decimal`               | DECIMAL                                              |
//!
//! ### [`uuid`](https://crates.io/crates/uuid)
//!
//! Requires the `uuid` Cargo feature flag.
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | `uuid::Uuid`                          | BINARY(16), VARCHAR, CHAR, TEXT                      |
//! | `uuid::fmt::Hyphenated`               | CHAR(36), UUID (MariaDB-only)                        |
//! | `uuid::fmt::Simple`                   | CHAR(32)                                             |
//!
//! ### [`json`](https://crates.io/crates/serde_json)
//!
//! Requires the `json` Cargo feature flag.
//!
//! | Rust type                             | MySQL/MariaDB type(s)                                |
//! |---------------------------------------|------------------------------------------------------|
//! | [`Json<T>`]                           | JSON                                                 |
//! | `serde_json::JsonValue`               | JSON                                                 |
//! | `&serde_json::value::RawValue`        | JSON                                                 |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from MySQL/MariaDB.

pub(crate) use sqlx_core::types::*;

mod bool;
mod bytes;
mod float;
mod inet;
mod int;
mod str;
mod text;
mod uint;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;

#[cfg(feature = "rust_decimal")]
mod rust_decimal;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "time")]
mod time;

#[cfg(feature = "uuid")]
mod uuid;
