#![cfg_attr(test, feature(test))]
#![allow(clippy::needless_lifetimes)]
#![allow(unused)]

#[cfg(test)]
extern crate test;

#[macro_use]
mod macros;

#[macro_use]
mod io;

mod backend;
pub mod deserialize;
mod url;

#[macro_use]
mod row;

mod connection;
pub mod error;
mod executor;
mod pool;

#[macro_use]
pub mod query;

pub mod serialize;
mod sql;
pub mod types;

#[doc(inline)]
pub use self::{
    backend::Backend,
    connection::Connection,
    deserialize::FromSql,
    error::{Error, Result},
    executor::Executor,
    pool::Pool,
    row::{FromSqlRow, Row},
    serialize::ToSql,
    sql::{query, SqlQuery},
    types::HasSqlType,
};

#[cfg(feature = "mariadb")]
pub mod mariadb;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use self::postgres::Postgres;
