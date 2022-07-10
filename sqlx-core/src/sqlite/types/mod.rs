//! Conversions between Rust and **SQLite** types.
//!
//! # Types
//!
//! | Rust type                             | SQLite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i8`                                  | INTEGER                                              |
//! | `i16`                                 | INTEGER                                              |
//! | `i32`                                 | INTEGER                                              |
//! | `i64`                                 | BIGINT, INT8                                         |
//! | `u8`                                  | INTEGER                                              |
//! | `u16`                                 | INTEGER                                              |
//! | `u32`                                 | INTEGER                                              |
//! | `f32`                                 | REAL                                                 |
//! | `f64`                                 | REAL                                                 |
//! | `&str`, [`String`]                    | TEXT                                                 |
//! | `&[u8]`, `Vec<u8>`                    | BLOB                                                 |
//!
//! #### Note: Unsigned Integers
//! The unsigned integer types `u8`, `u16` and `u32` are implemented by zero-extending to the
//! next-larger signed type. So `u8` becomes `i16`, `u16` becomes `i32`, and `u32` becomes `i64`
//! while still retaining their semantic values.
//!
//! Similarly, decoding performs a checked truncation to ensure that overflow does not occur.
//!
//! SQLite stores integers in a variable-width encoding and always handles them in memory as 64-bit
//! signed values, so no space is wasted by this implicit widening.
//!
//! However, there is no corresponding larger type for `u64` in SQLite (it would require a `i128`),
//! and so it is not supported. Bit-casting it to `i64` or storing it as `REAL`, `BLOB` or `TEXT`
//! would change the semantics of the value in SQL and so violates the principle of least surprise.
//!
//! ### [`chrono`](https://crates.io/crates/chrono)
//!
//! Requires the `chrono` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `chrono::NaiveDateTime`               | DATETIME                                             |
//! | `chrono::DateTime<Utc>`               | DATETIME                                             |
//! | `chrono::DateTime<Local>`             | DATETIME                                             |
//! | `chrono::NaiveDate`                   | DATE                                                 |
//! | `chrono::NaiveTime`                   | TIME                                                 |
//!
//! ### [`time`](https://crates.io/crates/time)
//!
//! Requires the `time` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `time::PrimitiveDateTime`             | DATETIME                                             |
//! | `time::OffsetDateTime`                | DATETIME                                             |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//!
//! ### [`uuid`](https://crates.io/crates/uuid)
//!
//! Requires the `uuid` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `uuid::Uuid`                          | BLOB, TEXT                                           |
//! | `uuid::fmt::Hyphenated`               | TEXT                                                 |
//!
//! ### [`json`](https://crates.io/crates/serde_json)
//!
//! Requires the `json` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | [`Json<T>`]                           | TEXT                                                 |
//! | `serde_json::JsonValue`               | TEXT                                                 |
//! | `&serde_json::value::RawValue`        | TEXT                                                 |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQLite.
//!

mod bool;
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
mod float;
mod int;
#[cfg(feature = "json")]
mod json;
mod str;
#[cfg(feature = "time")]
mod time;
mod uint;
#[cfg(feature = "uuid")]
mod uuid;
