//! SQLx is an async-first, pure Rust SQL crate featuring compile-time checked queries without a DSL.
//!
//! ## Database
//!
//! SQLx is **database agnostic**. Functionality is composed through traits defined in `sqlx-core` and implemented
//! in SQLx driver crates (`sqlx-postgres`, etc.). Enable support for a database by selecting an associated
//! crate feature.
//!
//! | Database | Supported Versions | Crate feature | Driver module |
//! | --- | --- | --- | --- |
//! | [MySQL](https://www.mysql.com/) | 5.0+, 8.0 | `mysql` | [`mysql`][mysql]
//! | [MariaDB](https://mariadb.com/) | 10.2+ | `mysql` | [`mysql`][mysql]
//!
//! ## Runtime
//!
//! SQLx is **asynchronous** (by default) and operation requires the selection of a runtime. This is done through
//! selecting one of the below crate features.
//!
//! Additionally, there is a **blocking** runtime available for simple use cases or environments where
//! asynchronous IO is either not available or not practical. The blocking runtime _does not wrap
//! the asynchronous runtime_. Blocking versions of the core traits are available in the [`blocking`] module.
//!
//! | Crate feature | Runtime
//! | --- | --- |
//! | `async-std` | [`AsyncStd`] |
//! | `tokio` | [`Tokio`] |
//! | `actix` | [`Actix`] |
//! | `blocking` | [`Blocking`] |
//!
#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(future_incompatible)]
#![warn(clippy::pedantic)]
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
#![allow(clippy::missing_errors_doc)]

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(feature = "pool")]
pub mod pool;

mod query;
mod query_as;
mod runtime;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "blocking")]
pub use blocking::Blocking;
pub use query::{query, Query};
pub use query_as::{query_as, QueryAs};
pub use runtime::DefaultRuntime;
#[cfg(feature = "actix")]
pub use sqlx_core::Actix;
#[cfg(feature = "async")]
pub use sqlx_core::Async;
#[cfg(feature = "async-std")]
pub use sqlx_core::AsyncStd;
#[cfg(feature = "tokio")]
pub use sqlx_core::Tokio;
pub use sqlx_core::{
    Acquire, Arguments, Close, Connect, ConnectOptions, Connection, Database, Decode, Describe,
    Encode, Error, Execute, Executor, FromRow, Null, Result, Row, Runtime, Type,
};
