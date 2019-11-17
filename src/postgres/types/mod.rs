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
use crate::types::{HasTypeMetadata, TypeMetadata};

mod binary;
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

impl HasTypeMetadata for Postgres {
    type TypeId = u32;
    type TypeMetadata = PostgresTypeMetadata;

    fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
        Some(match id {
            16 => "bool",
            1000 => "&[bool]",
            25 => "&str",
            1009 => "&[&str]",
            21 => "i16",
            1005 => "&[i16]",
            23 => "i32",
            1007 => "&[i32]",
            20 => "i64",
            1016 => "&[i64]",
            700 => "f32",
            1021 => "&[f32]",
            701 => "f64",
            1022 => "&[f64]",
            2950 => "sqlx::Uuid",
            2951 => "&[sqlx::Uuid]",
            _ => return None,
        })
    }

    fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
        Some(match id {
            16 => "bool",
            1000 => "Vec<bool>",
            25 => "String",
            1009 => "Vec<String>",
            21 => "i16",
            1005 => "Vec<i16>",
            23 => "i32",
            1007 => "Vec<i32>",
            20 => "i64",
            1016 => "Vec<i64>",
            700 => "f32",
            1021 => "Vec<f32>",
            701 => "f64",
            1022 => "Vec<f64>",
            2950 => "sqlx::Uuid",
            2951 => "Vec<sqlx::Uuid>",
            _ => return None,
        })
    }
}

impl TypeMetadata for PostgresTypeMetadata {
    type TypeId = u32;

    fn type_id(&self) -> &u32 {
        &self.oid
    }
}
