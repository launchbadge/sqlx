#![feature(non_exhaustive, async_await)]
#![allow(clippy::needless_lifetimes)]
// TODO: Remove this once API has matured
#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

pub mod connection;
pub mod protocol;
