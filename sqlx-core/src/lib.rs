//! SQLx Core (`sqlx-core`) is the core set of traits and types that are used and implemented for each
//! database driver (`sqlx-postgres`, `sqlx-mysql`, etc.).
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

mod acquire;
pub mod arguments;
mod close;
mod column;
mod connect;
mod connection;
pub mod database;
pub mod decode;
mod describe;
pub mod encode;
mod error;
mod execute;
mod executor;
mod from_row;
mod isolation_level;
mod null;
mod options;
mod query_result;
mod raw_value;
pub mod row;
mod runtime;
mod r#type;
mod type_info;

#[doc(hidden)]
pub mod io;

#[doc(hidden)]
pub mod net;

#[doc(hidden)]
pub mod placeholders;

#[doc(hidden)]
#[cfg(feature = "_mock")]
pub mod mock;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use acquire::Acquire;
pub use arguments::Arguments;
#[cfg(feature = "blocking")]
pub use blocking::runtime::Blocking;
pub use close::Close;
pub use column::Column;
pub use connect::Connect;
pub use connection::Connection;
pub use database::Database;
pub use decode::Decode;
pub use describe::Describe;
pub use encode::Encode;
pub use error::{ClientError, DatabaseError, Error, Result};
pub use execute::Execute;
pub use executor::Executor;
pub use from_row::FromRow;
pub use isolation_level::IsolationLevel;
pub use null::Null;
pub use options::ConnectOptions;
pub use query_result::QueryResult;
pub use r#type::{Type, TypeDecode, TypeDecodeOwned, TypeEncode};
pub use raw_value::RawValue;
pub use row::{ColumnIndex, Row};
#[cfg(feature = "actix")]
pub use runtime::Actix;
#[cfg(feature = "async")]
pub use runtime::Async;
#[cfg(feature = "async-std")]
pub use runtime::AsyncStd;
pub use runtime::Runtime;
#[cfg(feature = "tokio")]
pub use runtime::Tokio;
pub use type_info::TypeInfo;
