//! Conversions between Rust and MySQL types.
//!
//! Strict type checking is implemented according to the following tables when using
//! the type-checked query macros.
//!
//! Note that type conversions are not strict when used directly. As an example,
//! reading an `u8` from a `BIGINT` column will work (as long as the actual value
//! fits within `u8`, otherwise it would raise a decoding error).
//!
//! ## Types
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | `TINYINT(1)`, `BIT(1)`, `BOOLEAN`                    |
//! | `i8`                                  | `TINYINT`                                            |
//! | `i16`                                 | `SMALLINT`                                           |
//! | `i32`                                 | `INT`, `MEDIUMINT`                                   |
//! | `i64`                                 | `BIGINT`                                             |
//! | `u8`                                  | `TINYINT UNSIGNED`                                   |
//! | `u16`                                 | `SMALLINT UNSIGNED`                                  |
//! | `u32`                                 | `INT UNSIGNED`                                       |
//! | `u64`                                 | `BIGINT UNSIGNED`                                    |
//! | `f32`                                 | `FLOAT`                                              |
//! | `f64`                                 | `DOUBLE`                                             |
//! | `String`                              | `TEXT`, `VARCHAR`, `CHAR`                            |
//! | `Vec<u8>`                             | `BLOB`, `VARBINARY`, `BINARY`                        |
//!

mod bool;
mod str;
mod uint;
mod bytes;

// TODO: mod decimal;
// TODO: mod int;
// TODO: mod float;
// TODO: mod time;
// TODO: mod str;
// TODO: mod bytes;
// TODO: mod bit;
// TODO: mod uuid;
// TODO: mod json;
