#![feature(async_await)]
#![cfg_attr(test, feature(test))]
#![allow(clippy::needless_lifetimes)]
// FIXME: Remove this once API has matured
#![allow(dead_code, unused_imports, unused_variables)]

#[cfg(test)]
extern crate test;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_tryfrom_derive;

#[macro_use]
mod macros;

pub mod backend;
pub mod deserialize;

#[macro_use]
pub mod row;

pub mod serialize;
pub mod types;

#[cfg(feature = "mariadb")]
pub mod mariadb;

#[cfg(feature = "postgres")]
pub mod pg;

mod connection;
mod executor;
mod pool;
mod query;

pub use self::{
    connection::Connection,
    pool::Pool,
    query::{query, SqlQuery},
};

// TODO: Remove after Mariadb transitions to URIs
mod options;
