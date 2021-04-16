//! [PostgreSQL] database driver.
//!
//! [PostgreSQL]: https://www.postgresql.org/
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

use sqlx_core::Arguments;

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
// mod transaction;
mod type_id;
mod type_info;
pub mod types;

// #[cfg(test)]
// mod mock;

pub use column::PgColumn;
pub use connection::PgConnection;
pub use database::Postgres;
pub use error::{PgClientError, PgDatabaseError};
pub use options::PgConnectOptions;
pub use output::PgOutput;
pub use protocol::backend::{PgNotice, PgNoticeSeverity};
pub use query_result::PgQueryResult;
pub use raw_value::{PgRawValue, PgRawValueFormat};
pub use row::PgRow;
pub use type_id::PgTypeId;
pub use type_info::PgTypeInfo;

pub type PgArguments<'v> = Arguments<'v, Postgres>;
