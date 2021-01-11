//! Types and traits used to interact with a database driver
//! for **blocking** operations.
//!

mod acquire;
mod close;
mod connect;
mod connection;
mod options;
pub(crate) mod runtime;

pub use acquire::Acquire;
pub use close::Close;
pub use connect::Connect;
pub use connection::Connection;
pub use options::ConnectOptions;
pub use runtime::Runtime;

/// Convenience re-export of common traits for blocking operations.
pub mod prelude {
    #[doc(no_inline)]
    pub use super::Acquire as _;
    #[doc(no_inline)]
    pub use super::Close as _;
    #[doc(no_inline)]
    pub use super::Connect as _;
    #[doc(no_inline)]
    pub use super::ConnectOptions as _;
    #[doc(no_inline)]
    pub use super::Connection as _;
    #[doc(no_inline)]
    pub use super::Runtime as _;
    #[doc(no_inline)]
    pub use crate::Database as _;
}
