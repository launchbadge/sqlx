//! Types and traits used to implement a database driver with **blocking** I/O.
//!

mod connection;
mod options;
mod runtime;

pub use connection::Connection;
pub use options::ConnectOptions;
pub use runtime::{Blocking, Runtime};

pub mod prelude {
    pub use crate::Database as _;

    pub use super::ConnectOptions as _;
    pub use super::Connection as _;
    pub use super::Runtime as _;
}
