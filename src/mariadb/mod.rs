#![feature(non_exhaustive, async_await)]
#![allow(clippy::needless_lifetimes)]
// TODO: Remove this once API has matured
#![allow(dead_code, unused_imports, unused_variables)]

pub mod connection;
pub mod protocol;

#[macro_use]
pub mod macros;
