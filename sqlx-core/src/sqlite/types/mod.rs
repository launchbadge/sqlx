//! Conversions between Rust and **SQLite** types.
//!
//! # Types
//!
//! | Rust type                             | SQLite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i16`                                 | INTEGER                                              |
//! | `i32`                                 | INTEGER                                              |
//! | `i64`                                 | BIGINT, INT8                                         |
//! | `f32`                                 | REAL                                                 |
//! | `f64`                                 | REAL                                                 |
//! | `&str`, `String`                      | TEXT                                                 |
//! | `&[u8]`, `Vec<u8>`                    | BLOB                                                 |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQLite.
//!

// NOTE: all types are compatible with all other types in SQLite
//       so we explicitly opt-out of runtime type assertions by returning [true] for
//       all implementations of [Decode::accepts]

mod bool;
mod bytes;
mod float;
mod int;
mod str;
