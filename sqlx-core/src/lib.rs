//! Core of SQLx, the rust SQL toolkit. Not intended to be used directly.

// When compiling with support for SQLite we must allow some unsafe code in order to
// interface with the inherently unsafe C module. This unsafe code is contained
// to the sqlite module.
#![cfg_attr(feature = "sqlite", deny(unsafe_code))]
#![cfg_attr(not(feature = "sqlite"), forbid(unsafe_code))]
#![recursion_limit = "512"]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(test, feature = "bench"), feature(test))]
// #![warn(missing_docs)]

#[cfg(all(test, feature = "bench"))]
extern crate test;

// HACK: Allow a feature name the same name as a dependency
#[cfg(feature = "bigdecimal")]
extern crate bigdecimal_ as bigdecimal;

mod runtime;

#[macro_use]
pub mod error;

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[macro_use]
mod io;

pub mod connection;
pub mod cursor;
pub mod database;
pub mod value;

#[macro_use]
pub mod executor;

pub mod transaction;
mod url;

#[macro_use]
pub mod arguments;
pub mod decode;

#[doc(hidden)]
pub mod describe;

pub mod encode;
pub mod pool;
pub mod query;

#[macro_use]
pub mod query_as;

pub mod types;

#[macro_use]
pub mod row;

#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
#[macro_use]
mod logging;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod mysql;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;

#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub mod sqlite;

pub use error::{Error, Result};
