//! Conversions between Rust and **SQLite** types.
//!
//! # Types
//!
//! | Rust type                             | SQLite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i16`                                 | INTEGER                                              |
//! | `i32`                                 | INTEGER                                              |
//! | `i64`                                 | INTEGER                                              |
//! | `f32`                                 | REAL                                                 |
//! | `f64`                                 | REAL                                                 |
//! | `&str`, `String`                      | TEXT                                                 |
//! | `&[u8]`, `Vec<u8>`                    | BLOB                                                 |
//!
//! ### [`chrono`](https://crates.io/crates/chrono)
//!
//! Requires the `chrono` Cargo feature flag.
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `chrono::DateTime<Utc>`               | TIMESTAMP                                            |
//! | `chrono::DateTime<Local>`             | TIMETAMP                                             |
//! | `chrono::NaiveDateTime`               | DATETIME                                             |
//! | `chrono::NaiveDate`                   | DATE                                                 |
//! | `chrono::NaiveTime`                   | TIME                                                 |

//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQLite.
//!

use crate::decode::Decode;
use crate::sqlite::value::SqliteValue;
use crate::sqlite::Sqlite;

mod bool;
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
mod float;
mod int;
mod str;

impl<'de, T> Decode<'de, Sqlite> for Option<T>
where
    T: Decode<'de, Sqlite>,
{
    fn decode(value: SqliteValue<'de>) -> crate::Result<Self> {
        if value.is_null() {
            Ok(None)
        } else {
            <T as Decode<Sqlite>>::decode(value).map(Some)
        }
    }
}
