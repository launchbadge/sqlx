//! Conversions between Rust and [MySQL types](https://dev.mysql.com/doc/refman/8.0/en/data-types.html).
//!
//! Strict type checking is implemented according to the following tables when using
//! the type-checked query macros. The inferred type of an expression when using the strict
//! compile-time type-checking should be the first Rust type that appears below that matches
//! the SQL type.
//!
//! Note that type conversions are not strict when used directly. As an example,
//! reading an `u8` from a `BIGINT` column will work (as long as the actual value
//! fits within `u8`, otherwise it would raise a decoding error).
//!
//! Any required [crate features](https://doc.rust-lang.org/cargo/reference/features.html)
//! are shown next to the type.
//!
//! ## Integer Types
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | [`bool`][std::primitive::bool]        | `TINYINT(1)`<sup>[[1]](#1)</sup>, `BOOLEAN`          |
//! | [`i8`]                                | `TINYINT`                                            |
//! | [`i16`]                               | `SMALLINT`                                           |
//! | [`i32`]                               | `INT`, `MEDIUMINT`                                   |
//! | [`i64`]                               | `BIGINT`                                             |
//! | [`u8`]                                | `TINYINT UNSIGNED`                                   |
//! | [`u16`]                               | `SMALLINT UNSIGNED`                                  |
//! | [`u32`]                               | `INT UNSIGNED`, `MEDIUMINT UNSIGNED`                 |
//! | [`u64`]                               | `BIGINT UNSIGNED`                                    |
//!
//! 1. <a id="1"></a> The `BOOLEAN` type is an alias to `TINYINT(1)`. SQLx will recognize both and
//! infer the type to be `bool`.
//!
//! ## Fixed-Point Types
//!
//! | Rust type                  | MySQL type(s)                     | Crate feature               |
//! |----------------------------|-----------------------------------|-----------------------------|
//! | [`num_bigint::BigInt`]     | `DECIMAL(N, 0)`, `NUMERIC(N, 0)`  | `bigint`                    |
//! | [`bigdecimal::BigDecimal`] | `DECIMAL`, `NUMERIC`              | `bigdecimal`                |
//! | [`rust_decimal::Decimal`]  | `DECIMAL`, `NUMERIC`              | `decimal`                   |
//!
//! ## Floating-Point Types
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | [`f32`]                               | `FLOAT`                                              |
//! | [`f64`]                               | `DOUBLE`, `REAL`                                     |
//!
//! ## Bit-Value Type - `BIT`
//!
//! | Rust type                             | MySQL type(s)         | Crate feature                |
//! |---------------------------------------|-----------------------|------------------------------|
//! | [`bool`]                              | `BIT(1)`              |                              |
//! | [`bitvec::BitVec`]                    | `BIT`                 | `bitvec`                     |
//! | [`u64`]                               | `BIT`                 |                              |
//!
//! ## String Types
//!
//! | Rust type                                      | MySQL type(s)                               |
//! |------------------------------------------------|---------------------------------------------|
//! | [`String`], [`&'r str`][&str]                  | `TEXT`, `VARCHAR`, `CHAR`                   |
//! | [`bytestring::ByteString`]<sup>[[2]](#2)</sup> | `TEXT`, `VARCHAR`, `CHAR`                   |
//!
//! ## Binary String Types
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | [`Vec<u8>`], [`&'r [u8]`][slice]      | `BLOB`, `VARBINARY`, `BINARY`                        |
//! | [`bytes::Bytes`]<sup>[[2]](#2)</sup>  | `BLOB`, `VARBINARY`, `BINARY`                        |
//!
//! 2. <a id="2"></a> The `Bytes` and `ByteString` types can be used as zero-copy containers to
//!     read binary and textual data from the connection.
//!
//! ## Date and Time Types
//!
//! | Rust type                             | MySQL type(s)         | Crate feature                |
//! |---------------------------------------|-----------------------|------------------------------|
//! | [`u16`]                               | `YEAR`                |                              |
//! | [`time::Date`]                        | `DATE`                | `time`                       |
//! | [`time::Time`]                        | `TIME`                | `time`                       |
//! | [`time::PrimitiveDateTime`]           | `DATETIME`            | `time`                       |
//! | [`time::OffsetDateTime`]              | `TIMESTAMP`           | `time`                       |
//! | [`chrono::NaiveDate`]                 | `DATE`                | `chrono`                     |
//! | [`chrono::NaiveTime`]                 | `TIME`                | `chrono`                     |
//! | [`chrono::NaiveDateTime`]             | `DATETIME`            | `chrono`                     |
//! | [`chrono::DateTime<Utc>`]             | `TIMESTAMP`           | `chrono`                     |
//! | [`chrono::DateTime<Local>`]           | `TIMESTAMP`           | `chrono`                     |
//! | [`chrono::DateTime<FixedOffset>`]     | `TIMESTAMP`           | `chrono`                     |
//!
//! ## JSON Type - `JSON`
//!
//! | Rust type                             | MySQL type(s)                      | Crate feature   |
//! |---------------------------------------|------------------------------------|-----------------|
//! | [`serde_json::Value`]                 | `JSON`<sup>[[3]](#3)</sup>, `TEXT` | `json`          |
//! | [`&'r serde_json::value::RawValue`]   | `JSON`<sup>[[3]](#3)</sup>, `TEXT` | `json`          |
//! | [`sqlx::types::Json<T>`]              | `JSON`<sup>[[3]](#3)</sup>, `TEXT` | `json`          |
//!
//! 3. <a id="3"></a> The `JSON` SQL type is supported by MySQL 8+ **only** (not in MariaDB). To
//!     use `JSON` in MariaDB or older MySQL versions, SQLx also supports any
//!     string type (eg., `TEXT`).
//!
//! ## UUID Type
//!
//! | Rust type                     | MySQL type(s)            | Crate feature |
//! |-------------------------------|--------------------------|---------------|
//! | [`uuid::Uuid`]                | `BINARY(16)`, `CHAR(32)` | `uuid`        |
//! | [`uuid::adapter::Hyphenated`] | `CHAR(36)`               | `uuid`        |
//!
//! ## Nullable Type
//!
//! | Rust type                                      | MySQL type(s)           |
//! |------------------------------------------------|-------------------------|
//! | [`Option<T>`]<sup>[[4]](#4)</sup>              | (any)                   |
//! | [`sqlx::types::Null`]<sup>[[5]](#5)</sup>      | (any)                   |
//!
//! 4. <a id="4"></a> The `Option<T>` type represents a potentially `NULL`
//!     value. Use this anywhere that you _might_ receive a `NULL`. The compile-time
//!     type-checking will enforce using this where necessary.
//!
//! 5. <a id="4"></a> The `Null` type represents a value that is _always_ `NULL`. This
//!     can be useful when you wish to pass a `NULL` as a parameter without knowing (or
//!     caring about the actual type of the parameter).
//!

mod bool;
mod bytes;
mod int;
mod null;
mod str;
mod uint;

// TODO: mod decimal;
// TODO: mod float;
// TODO: mod time;
// TODO: mod bit;
// TODO: mod uuid;
// TODO: mod json;
