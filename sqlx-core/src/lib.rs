//! Core of SQLx, the rust SQL toolkit. Not intended to be used directly.

#![forbid(unsafe_code)]
#![recursion_limit = "512"]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(unused)]

#[macro_use]
pub mod error;

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[macro_use]
mod io;

pub mod connection;
pub mod cursor;
pub mod database;

#[macro_use]
pub mod executor;

pub mod transaction;
mod url;

#[doc(hidden)]
pub mod runtime;

#[macro_use]
pub mod arguments;
pub mod decode;
pub mod describe;
pub mod encode;
pub mod pool;
pub mod query;

#[macro_use]
pub mod query_as;

pub mod types;

#[macro_use]
pub mod row;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod mysql;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;

pub use error::{Error, Result};

// Named Lifetimes:
//  'c: connection
//  'q: query string (and arguments)
