//! Traits linking Rust types to SQL types.

pub use sqlx_core::types::*;

#[cfg(feature = "macros")]
pub use sqlx_macros::HasSqlType;
