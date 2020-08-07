//! Core of SQLx, the rust SQL toolkit. Not intended to be used directly.
#![deny(unsafe_code)]
#![warn(future_incompatible, rust_2018_idioms, unreachable_pub)]

pub mod cache;
pub mod connection;
pub mod database;
pub mod error;
pub mod execute;
pub mod executor;
pub mod arguments;
pub mod io;
pub mod options;
pub mod type_info;
pub mod query;
pub mod to_value;

// there are several consistently named lifetimes used
// throughout this project:

// 'e = Executor
// 'r = Row
// 'c = Connection
// 'p = Pool
// 'x = Execution
// 'q = Query
