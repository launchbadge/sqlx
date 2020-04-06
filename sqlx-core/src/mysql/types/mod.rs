//! Conversions between Rust and **MySQL** types.
//!
//! # Types
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | TINYINT(1)                                           |
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
//! | `&str`, `String`                      | VARCHAR, CHAR, TEXT                                  |
//! | `&[u8]`, `Vec<u8>`                    | VARBINARY, BINARY, BLOB                              |
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
//! ### [`time`](https://crates.io/crates/time)
//!
//! Requires the `time` Cargo feature flag.
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `time::PrimitiveDateTime`             | DATETIME                                             |
//! | `time::OffsetDateTime`                | TIMESTAMP                                            |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//!
//! ### [`bigdecimal`](https://crates.io/crates/bigdecimal)
//! Requires the `bigdecimal` Cargo feature flag.
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `bigdecimal::BigDecimal`              | DECIMAL                                              |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from MySQL.
//!

mod bool;
mod bytes;
mod float;
mod int;
mod str;
mod uint;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "time")]
mod time;

use crate::decode::Decode;
use crate::mysql::{MySql, MySqlValue};

impl<'de, T> Decode<'de, MySql> for Option<T>
where
    T: Decode<'de, MySql>,
{
    fn decode(value: MySqlValue<'de>) -> crate::Result<Self> {
        Ok(if value.get().is_some() {
            Some(<T as Decode<MySql>>::decode(value)?)
        } else {
            None
        })
    }
}
