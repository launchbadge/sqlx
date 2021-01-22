//! [PostgreSQL] database driver.
//!
//! [PostgreSQL]: https://www.postgres.com/
//!
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(not(any(feature = "async", feature = "blocking")), allow(unused))]
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

mod connection;
mod database;
mod error;
mod io;
mod options;
mod protocol;

#[cfg(test)]
mod mock;

pub use connection::PostgresConnection;
pub use database::Postgres;
pub use error::PostgresDatabaseError;
pub use options::PostgresConnectOptions;
