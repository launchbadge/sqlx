#![forbid(unsafe_code)]
#![allow(unused)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
pub mod error;

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[macro_use]
mod io;

#[cfg(any(feature = "mysql", feature = "postgres"))]
mod cache;

mod connection;
mod cursor;
mod database;

#[macro_use]
mod executor;

mod transaction;
mod url;

#[doc(hidden)]
pub mod runtime;

#[macro_use]
pub mod arguments;

#[doc(hidden)]
pub mod decode;

pub mod describe;
pub mod encode;
pub mod pool;
pub mod query;
pub mod types;

#[macro_use]
pub mod row;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod mysql;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;

pub use database::Database;

#[doc(inline)]
pub use error::{Error, Result};

pub use connection::{Connect, Connection};
pub use cursor::Cursor;
pub use executor::{Execute, Executor};
pub use transaction::Transaction;

#[doc(inline)]
pub use pool::Pool;

#[doc(inline)]
pub use row::{FromRow, Row};

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
#[doc(inline)]
pub use mysql::MySql;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
#[doc(inline)]
pub use postgres::Postgres;

// Named Lifetimes:
//  'c: connection
//  'q: query string (and arguments)
