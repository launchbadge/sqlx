//! Types and traits used to implement a database driver with **blocking** I/O.
//!

mod acquire;
mod close;
mod connect;
mod connection;
mod options;
mod runtime;

pub use acquire::Acquire;
pub use close::Close;
pub use connect::Connect;
pub use connection::Connection;
pub use options::ConnectOptions;
pub use runtime::{Blocking, Runtime};

pub mod prelude {
    pub use super::Acquire as _;
    pub use super::Close as _;
    pub use super::Connect as _;
    pub use super::ConnectOptions as _;
    pub use super::Connection as _;
    pub use super::Runtime as _;
    pub use crate::Database as _;
}
