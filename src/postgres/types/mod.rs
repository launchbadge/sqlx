//! PostgreSQL types.
//!
//! The following types are supported by this crate,
//! along with the corresponding Postgres types:
//!
//! ### Standard
//!
//! | Rust type                         | Postgres type(s)                     |
//! |-----------------------------------|--------------------------------------|
//! | `i16`                             | SMALLINT, SMALLSERIAL                |
//! | `i32`                             | INT, SERIAL                          |
//! | `i64`                             | BIGINT, BIGSERIAL                    |
//! | `f32`                             | REAL                                 |
//! | `f64`                             | DOUBLE PRECISION                     |
//! | `&str`/`String`                   | VARCHAR, CHAR(n), TEXT, CITEXT, NAME |
//! | `&[u8]`/`Vec<u8>`                 | BYTEA                                |
//!
//! ### PostgreSQL specific
//!
//! | Rust type                         | Postgres type(s)                     |
//! |-----------------------------------|--------------------------------------|
//! | `bool`                            | BOOL                                 |
//! | `i8`                              | "char"                               |
//! | `u32`                             | OID                                  |
//! | `&[u8]`/`Vec<u8>`                 | BYTEA                                |
//! | `HashMap<String, Option<String>>` | HSTORE                               |
//! | `IpAddr`                          | INET                                 |
//! | `Uuid` (`uuid` feature)           | UUID                                 |

use super::Postgres;
use crate::types::TypeMetadata;
use crate::HasSqlType;

mod boolean;
mod character;
mod numeric;

#[cfg(feature = "uuid")]
mod uuid;

pub enum PostgresTypeFormat {
    Text = 0,
    Binary = 1,
}

/// Provides the OIDs for a SQL type and the expected format to be used for
/// transmission between Rust and PostgreSQL.
///
/// While the BINARY format is preferred in most cases, there are scenarios
/// where only the TEXT format may be available for a type.
pub struct PostgresTypeMetadata {
    pub format: PostgresTypeFormat,
    pub oid: u32,
    pub array_oid: u32,
}

impl TypeMetadata for Postgres {
    type TypeMetadata = PostgresTypeMetadata;
}
