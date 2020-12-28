//! SQLx Core (`sqlx-core`) is the core set of traits and types that are used and implemented for each
//! database driver (`sqlx-postgres`, `sqlx-mysql`, etc.).
//!
#![cfg_attr(doc_cfg, feature(doc_cfg))]
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

// crate renames to allow the feature name "tokio" and "async-std" (as features
// can't directly conflict with dependency names)

#[cfg(feature = "async-std")]
extern crate _async_std as async_std;

#[cfg(feature = "tokio")]
extern crate _tokio as tokio;

mod connection;
mod database;
mod error;
mod options;
mod runtime;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use connection::Connection;
pub use database::{Database, HasOutput};
pub use error::{Error, Result};
pub use options::ConnectOptions;
pub use runtime::Runtime;

#[cfg(feature = "async-std")]
pub use runtime::AsyncStd;

#[cfg(feature = "tokio")]
pub use runtime::Tokio;

#[cfg(feature = "actix")]
pub use runtime::Actix;

#[cfg(feature = "async")]
pub use runtime::Async;

#[cfg(feature = "async-std")]
pub type DefaultRuntime = AsyncStd;

#[cfg(all(not(feature = "async-std"), feature = "tokio"))]
pub type DefaultRuntime = Tokio;

#[cfg(all(not(all(feature = "async-std", feature = "tokio")), feature = "actix"))]
pub type DefaultRuntime = Actix;

#[cfg(not(feature = "async"))]
pub type DefaultRuntime = blocking::Blocking;

pub mod prelude {
    pub use super::ConnectOptions as _;
    pub use super::Connection as _;
    pub use super::Database as _;
    pub use super::Runtime as _;

    #[cfg(all(not(feature = "async"), feature = "blocking"))]
    pub use super::blocking::prelude::*;
}
