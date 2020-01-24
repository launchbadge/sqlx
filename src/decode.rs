//! Types and traits for decoding values from the database.

pub use sqlx_core::decode::*;

#[cfg(feature = "macros")]
pub use sqlx_macros::Decode;
