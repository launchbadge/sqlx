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
mod column;
mod connection;
mod database;
mod io;
mod options;
mod statement;
mod type_info;
pub mod types;

pub use column::PgColumn;
pub use connection::PgConnection;
pub use database::Postgres;
pub use options::{PgConnectOptions, PgSslMode};
pub use statement::PgStatement;
pub use type_info::{PgTypeId, PgTypeInfo, PgTypeKind};
