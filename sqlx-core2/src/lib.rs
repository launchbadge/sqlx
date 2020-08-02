//! Core of SQLx, the rust SQL toolkit. Not intended to be used directly.
#![deny(unsafe_code)]
#![warn(future_incompatible, rust_2018_idioms, unreachable_pub)]
pub mod connection;
pub mod database;
pub mod error;
pub mod io;
pub mod options;
