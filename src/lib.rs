#![cfg_attr(test, feature(test))]
#![allow(clippy::needless_lifetimes)]
#![allow(unused)]

// #[macro_use]
// extern crate bitflags;

#[macro_use]
extern crate bitflags;

#[cfg(test)]
extern crate test;

#[macro_use]
mod macros;

#[macro_use]
mod io;

pub mod backend;
pub mod deserialize;

#[macro_use]
pub mod row;

pub mod serialize;
pub mod types;

#[cfg(feature = "mariadb")]
pub mod mariadb;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use self::postgres::Postgres;

mod connection;
pub mod error;
mod executor;
mod pool;
mod query;

pub use self::{
    connection::Connection,
    error::Error,
    pool::Pool,
    query::{query, SqlQuery},
};
