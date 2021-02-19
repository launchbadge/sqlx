//! [MySQL] database driver.
//!
//! [MySQL]: https://www.mysql.com/
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
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

#[macro_use]
mod stream;

mod column;
mod connection;
mod database;
mod error;
mod io;
mod options;
mod output;
mod protocol;
mod query_result;
mod raw_statement;
mod raw_value;
mod row;
mod type_id;
mod type_info;
pub mod types;

#[cfg(test)]
mod mock;

pub use column::MySqlColumn;
pub use connection::MySqlConnection;
pub use database::MySql;
pub use error::MySqlDatabaseError;
pub use options::MySqlConnectOptions;
pub use output::MySqlOutput;
pub use query_result::MySqlQueryResult;
pub use raw_value::{MySqlRawValue, MySqlRawValueFormat};
pub use row::MySqlRow;
pub use type_id::MySqlTypeId;
pub use type_info::MySqlTypeInfo;
