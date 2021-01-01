//! SQLx is an async, pure Rust SQL crate featuring compile-time checked queries without a DSL.
//!
#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(future_incompatible)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo_common_metadata)]
#![warn(clippy::multiple_crate_versions)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::future_not_send)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::redundant_pub_crate)]
#![warn(clippy::string_lit_as_bytes)]
#![warn(clippy::use_self)]
#![warn(clippy::useless_let_if_seq)]
#![allow(clippy::doc_markdown)]

#[cfg(feature = "blocking")]
pub use sqlx_core::blocking;
#[cfg(feature = "actix")]
pub use sqlx_core::Actix;
#[cfg(feature = "async-std")]
pub use sqlx_core::AsyncStd;
#[cfg(feature = "tokio")]
pub use sqlx_core::Tokio;
pub use sqlx_core::{
    prelude, ConnectOptions, Connection, Database, DefaultRuntime, Error, Result, Runtime,
};
#[cfg(feature = "mysql")]
pub use sqlx_mysql as mysql;
