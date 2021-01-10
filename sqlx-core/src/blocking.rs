//! Types and traits used to implement a database driver with **blocking** I/O.
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

pub mod prelude {
    pub use super::Acquire;
    pub use super::Close;
    pub use super::Connect;
    pub use super::ConnectOptions;
    pub use super::Connection;
    pub use super::Runtime;
    pub use crate::Database;
}

/// Uses the `std::net` primitives to implement a blocking runtime for SQLx.
#[derive(Debug)]
pub struct Blocking;

impl crate::Runtime for Blocking {
    type TcpStream = TcpStream;
}

impl Runtime for Blocking {
    fn connect_tcp(host: &str, port: u16) -> IoResult<Self::TcpStream> {
        TcpStream::connect((host, port))
    }
}

// 's: stream
impl<'s> crate::io::Stream<'s, Blocking> for TcpStream {
    #[cfg(feature = "async")]
    type ReadFuture = futures_util::future::BoxFuture<'s, IoResult<usize>>;

    #[cfg(feature = "async")]
    type WriteFuture = futures_util::future::BoxFuture<'s, IoResult<usize>>;

    #[cfg(feature = "async")]
    fn read_async(&'s mut self, _buf: &'s mut [u8]) -> Self::ReadFuture {
        // UNREACHABLE: [`Blocking`] does not implement the [`Async`] marker
        unreachable!()
    }

    #[cfg(feature = "async")]
    fn write_async(&'s mut self, _buf: &'s [u8]) -> Self::WriteFuture {
        // UNREACHABLE: [`Blocking`] does not implement the [`Async`] marker
        unreachable!()
    }
}

// 's: stream
impl<'s> io::Stream<'s, Blocking> for TcpStream {
    fn read(&'s mut self, buf: &'s mut [u8]) -> IoResult<usize> {
        Read::read(self, buf)
    }

    fn write(&'s mut self, buf: &'s [u8]) -> IoResult<usize> {
        let size = buf.len();
        self.write_all(buf)?;

        Ok(size)
    }
}
