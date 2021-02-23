//! Types and traits used to interact with a database driver
//! for **blocking** operations.
//!

mod query;
mod query_as;
mod runtime;
mod stream;

// [Blocking] is wrapped from [sqlx_core] instead of re-exporting so
// we can use crate-local negative inference to allow inherent impls
// for [DbConnection<Blocking>] **and** [DbConnection<Rt> where Rt: Async]

pub use runtime::Blocking;
pub use sqlx_core::blocking::{
    Acquire, Close, Connect, ConnectOptions, Connection, Executor, Runtime,
};
