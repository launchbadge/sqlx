//! **PostgreSQL** database driver.
//!
#![forbid(unsafe_code)]
#![warn(
    future_incompatible,
    rust_2018_idioms,
    missing_docs,
    missing_doc_code_examples,
    unreachable_pub
)]
#![allow(unused)]

mod codec;
mod connection;
mod database;
mod io;
mod options;

pub use connection::PgConnection;
pub use database::Postgres;
pub use options::{PgConnectOptions, PgSslMode};
