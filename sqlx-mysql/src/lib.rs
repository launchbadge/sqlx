//! [MySQL] database driver for [SQLx][sqlx_core], the Rust SQL toolkit.
//!
//! [MySQL]: https://www.mysql.com/
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

mod connection;
mod database;
mod options;

#[cfg(feature = "blocking")]
mod blocking;

pub use connection::MySqlConnection;
pub use database::MySql;
pub use options::MySqlConnectOptions;
