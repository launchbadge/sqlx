//! Types and traits used to interact with a database driver
//! for **blocking** operations.
//!

use std::io::{Read, Result as IoResult, Write};
use std::net::TcpStream;

mod acquire;
mod close;
mod connect;
mod connection;
mod options;
mod runtime;

#[doc(hidden)]
pub mod io;

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

pub(super) mod rt {
    /// Uses the `std::net` primitives to implement a blocking runtime for SQLx.
    #[derive(Debug)]
    pub struct Blocking;
}

impl crate::Runtime for rt::Blocking {
    #[doc(hidden)]
    type TcpStream = TcpStream;
}

impl Runtime for rt::Blocking {
    #[doc(hidden)]
    fn connect_tcp(host: &str, port: u16) -> IoResult<Self::TcpStream> {
        TcpStream::connect((host, port))
    }
}

// 's: stream
impl<'s> crate::io::Stream<'s, rt::Blocking> for TcpStream {
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = futures_util::future::BoxFuture<'s, IoResult<usize>>;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = futures_util::future::BoxFuture<'s, IoResult<usize>>;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn read_async(&'s mut self, _buf: &'s mut [u8]) -> Self::ReadFuture {
        // UNREACHABLE: [`Blocking`] does not implement the [`Async`] marker
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn write_async(&'s mut self, _buf: &'s [u8]) -> Self::WriteFuture {
        // UNREACHABLE: [`Blocking`] does not implement the [`Async`] marker
        unreachable!()
    }
}

// 's: stream
impl<'s> io::Stream<'s, rt::Blocking> for TcpStream {
    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> IoResult<usize> {
        Read::read(self, buf)
    }

    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> IoResult<usize> {
        let size = buf.len();
        self.write_all(buf)?;

        Ok(size)
    }
}
