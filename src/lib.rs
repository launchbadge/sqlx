#![feature(non_exhaustive, async_await)]
#![cfg_attr(test, feature(test))]

#![allow(clippy::needless_lifetimes)]

#[cfg(test)]
extern crate test;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_tryfrom_derive;

mod options;
pub use self::options::ConnectOptions;

pub mod postgres;
pub mod mariadb;
