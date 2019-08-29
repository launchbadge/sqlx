#![cfg_attr(test, feature(test))]
#![allow(clippy::needless_lifetimes)]
#![allow(unused)]

#[cfg(test)]
extern crate test;

#[macro_use]
mod macros;

#[macro_use]
mod io;

pub mod backend;
pub mod deserialize;
mod url;

#[macro_use]
pub mod row;

mod connection;
pub mod error;
mod executor;
mod pool;

#[macro_use]
pub mod query;

pub mod serialize;
mod sql;
pub mod types;

pub use self::{
    connection::Connection,
    error::Error,
    executor::Executor,
    pool::Pool,
    sql::{query, SqlQuery},
};

#[cfg(feature = "mariadb")]
pub mod mariadb;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use self::postgres::Postgres;
