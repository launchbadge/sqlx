//! SQLx Core (`sqlx-core`) is the core set of traits and types that are used and implemented for each
//! database driver (`sqlx-postgres`, `sqlx-mysql`, etc.).
//!
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(future_incompatible)]
#![warn(clippy::pedantic)]
#![warn(clippy::multiple_crate_versions)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::future_not_send)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::string_lit_as_bytes)]
#![warn(clippy::use_self)]
#![warn(clippy::useless_let_if_seq)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::clippy::missing_errors_doc)]

mod acquire;
mod close;
mod connect;
mod connection;
mod database;
mod error;
mod options;
mod pool;
mod runtime;

#[doc(hidden)]
pub mod io;

#[doc(hidden)]
#[cfg(feature = "_mock")]
pub mod mock;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use acquire::Acquire;
#[cfg(feature = "blocking")]
pub use blocking::rt::Blocking;
pub use close::Close;
pub use connect::Connect;
pub use connection::Connection;
pub use database::{Database, HasOutput};
pub use error::{DatabaseError, Error, Result};
pub use options::ConnectOptions;
pub use pool::Pool;
#[cfg(feature = "actix")]
pub use runtime::Actix;
#[cfg(feature = "async-std")]
pub use runtime::AsyncStd;
#[cfg(feature = "tokio")]
pub use runtime::Tokio;
pub use runtime::{Async, DefaultRuntime, Runtime};

/// Convenience re-export of common traits for non-blocking operations.
pub mod prelude {
    #[doc(no_inline)]
    pub use super::Acquire as _;
    #[doc(no_inline)]
    pub use super::Close as _;
    #[doc(no_inline)]
    pub use super::Connect as _;
    #[doc(no_inline)]
    pub use super::ConnectOptions as _;
    #[doc(no_inline)]
    pub use super::Connection as _;
    #[doc(no_inline)]
    pub use super::Database as _;
    #[doc(no_inline)]
    pub use super::Runtime as _;
}

#[cfg(all(
    not(any(feature = "async-std", feature = "tokio", feature = "actix")),
    feature = "blocking"
))]
pub use blocking::prelude;
