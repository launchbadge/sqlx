//! Types and traits used to interact with a database driver
//! for **blocking** operations.
//!

mod acquire;
mod close;
mod connect;
mod connection;
mod options;
mod executor;
pub(crate) mod runtime;

pub use executor::Executor;
pub use acquire::Acquire;
pub use close::Close;
pub use connect::Connect;
pub use connection::Connection;
pub use options::ConnectOptions;
pub use runtime::Runtime;
