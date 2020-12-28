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
#![allow(clippy::clippy::doc_markdown)]

// crate renames to allow the feature name "tokio" and "async-std" (as features
// can't directly conflict with dependency names)

#[cfg(feature = "async-std")]
extern crate _async_std as async_std;

#[cfg(feature = "tokio")]
extern crate _tokio as tokio;

#[cfg(feature = "async")]
mod runtime;

#[cfg(feature = "blocking")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "blocking")))]
pub mod blocking;

#[cfg(feature = "blocking")]
pub use blocking::runtime::Blocking;

#[cfg(feature = "async")]
pub use runtime::Runtime;

#[cfg(all(feature = "async", feature = "async-std"))]
pub use runtime::async_std::AsyncStd;

#[cfg(all(feature = "async", feature = "tokio"))]
pub use runtime::tokio::Tokio;

#[cfg(all(feature = "async", feature = "actix"))]
pub use runtime::actix::Actix;
