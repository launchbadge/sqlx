#![feature(non_exhaustive, async_await)]
#![allow(clippy::needless_lifetimes)]

mod options;
pub use self::options::ConnectOptions;

pub mod postgres;
