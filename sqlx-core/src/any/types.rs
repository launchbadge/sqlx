//! Conversions between Rust and standard **SQL** types.
//!
//! # Types
//!
//! | Rust type                             | SQL type(s)                                          |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i16`                                 | SMALLINT                                             |
//! | `i32`                                 | INT                                                  |
//! | `i64`                                 | BIGINT                                               |
//! | `f32`                                 | FLOAT                                                |
//! | `f64`                                 | DOUBLE                                               |
//! | `&str`, [`String`]                    | VARCHAR, CHAR, TEXT                                  |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQL.
//!

// Type

impl_any_type!(bool);

impl_any_type!(i16);
impl_any_type!(i32);
impl_any_type!(i64);

impl_any_type!(f32);
impl_any_type!(f64);

impl_any_type!(str);
impl_any_type!(String);

// Encode

impl_any_encode!(bool);

impl_any_encode!(i16);
impl_any_encode!(i32);
impl_any_encode!(i64);

impl_any_encode!(f32);
impl_any_encode!(f64);

impl_any_encode!(&'q str);
impl_any_encode!(String);

// Decode

impl_any_decode!(bool);

impl_any_decode!(i16);
impl_any_decode!(i32);
impl_any_decode!(i64);

impl_any_decode!(f32);
impl_any_decode!(f64);

impl_any_decode!(&'r str);
impl_any_decode!(String);

// Conversions for Blob SQL types
// Type
#[cfg(all(
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_type!([u8]);
#[cfg(all(
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_type!(Vec<u8>);

// Encode
#[cfg(all(
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_encode!(&'q [u8]);
#[cfg(all(
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_encode!(Vec<u8>);

// Decode
#[cfg(all(
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_decode!(&'r [u8]);
#[cfg(all(
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_decode!(Vec<u8>);

// Conversions for Time SQL types
// Type
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_type!(chrono::NaiveDate);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_type!(chrono::NaiveTime);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_type!(chrono::NaiveDateTime);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_type!(chrono::DateTime<chrono::offset::Utc>);
#[cfg(all(
    feature = "chrono",
    any(feature = "sqlite", feature = "postgres", feature = "mysql"),
    not(feature = "mssql")
))]
impl_any_type!(chrono::DateTime<chrono::offset::Local>);

// Encode
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_encode!(chrono::NaiveDate);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_encode!(chrono::NaiveTime);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_encode!(chrono::NaiveDateTime);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_encode!(chrono::DateTime<chrono::offset::Utc>);
#[cfg(all(
    feature = "chrono",
    any(feature = "sqlite", feature = "postgres", feature = "mysql"),
    not(feature = "mssql")
))]
impl_any_encode!(chrono::DateTime<chrono::offset::Local>);

// Decode
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_decode!(chrono::NaiveDate);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_decode!(chrono::NaiveTime);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_decode!(chrono::NaiveDateTime);
#[cfg(all(
    feature = "chrono",
    any(feature = "mysql", feature = "sqlite", feature = "postgres"),
    not(feature = "mssql")
))]
impl_any_decode!(chrono::DateTime<chrono::offset::Utc>);
#[cfg(all(
    feature = "chrono",
    any(feature = "sqlite", feature = "postgres", feature = "mysql"),
    not(feature = "mssql")
))]
impl_any_decode!(chrono::DateTime<chrono::offset::Local>);
