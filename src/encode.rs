//! Types and traits for encoding values to the database.

pub use sqlx_core::encode::*;

#[cfg(feature = "macros")]
pub use sqlx_macros::Encode;
