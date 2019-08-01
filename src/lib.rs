#![feature(non_exhaustive, async_await, async_closure)]
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

mod options;
pub use self::options::ConnectOptions;

pub mod mariadb;
pub mod postgres;

// Helper macro for writing long complex tests
#[macro_use]
pub mod macros;

pub mod row;
pub mod types;
