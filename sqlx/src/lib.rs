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
//! | [MySQL](https://www.mysql.com/) | 5.0+, 8.0 | `mysql` | [`mysql`][sqlx_mysql]
//! | [MariaDB](https://mariadb.com/) | 10.2+ | `mysql` | [`mysql`][sqlx_mysql]
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
//! | Crate feature | Runtime | Prelude |
//! | --- | --- | --- |
//! | `async-std` | [`AsyncStd`] | [`sqlx::prelude`][prelude] or [`sqlx::blocking::prelude`][blocking::prelude] |
//! | `tokio` | [`Tokio`] | [`sqlx::prelude`][prelude]|
//! | `actix` | [`Actix`] | [`sqlx::prelude`][prelude] |
//! | `blocking` | [`Blocking`] | [`sqlx::blocking::prelude`][blocking::prelude] |
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

#[cfg(feature = "actix")]
pub use sqlx_core::Actix;
#[cfg(feature = "async-std")]
pub use sqlx_core::AsyncStd;
#[cfg(feature = "tokio")]
pub use sqlx_core::Tokio;
#[cfg(feature = "blocking")]
pub use sqlx_core::{blocking, Blocking};
pub use sqlx_core::{
    prelude, Acquire, Close, Connect, ConnectOptions, Connection, Database, DefaultRuntime, Error,
    Result, Runtime,
};
#[cfg(feature = "mysql")]
#[doc(inline)]
pub use sqlx_mysql as mysql;
