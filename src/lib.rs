<<<<<<< HEAD
#![feature(non_exhaustive, async_await)]
#![cfg_attr(test, feature(test))]

#![allow(clippy::needless_lifetimes)]

#[cfg(test)]
extern crate test;

mod options;
pub use self::options::ConnectOptions;

pub mod postgres;
