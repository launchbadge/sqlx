//! Types and traits used to implement a database driver with **blocking** I/O.
//!

pub(crate) mod runtime;

mod connection;
mod options;

pub use connection::Connection;
pub use options::ConnectOptions;
pub use runtime::Runtime;
