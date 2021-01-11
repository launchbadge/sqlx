use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::Path;

#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

use crate::io::Stream as IoStream;

/// Marks a [`Runtime`][crate::Runtime] as being capable of executing blocking operations.
pub trait Runtime: crate::Runtime {}

/// Uses the `std::net` primitives to implement a blocking runtime for SQLx.
#[derive(Debug)]
pub struct Blocking;

impl crate::Runtime for Blocking {
    #[doc(hidden)]
    type TcpStream = TcpStream;

    #[doc(hidden)]
    #[cfg(unix)]
    type UnixStream = UnixStream;

    #[doc(hidden)]
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream> {
        TcpStream::connect((host, port))
    }

    #[doc(hidden)]
    #[cfg(all(unix, feature = "blocking"))]
    fn connect_unix(path: &Path) -> io::Result<Self::UnixStream> {
        UnixStream::connect(path)
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    #[allow(unused_variables)]
    fn connect_tcp_async(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(all(unix, feature = "async"))]
    #[allow(unused_variables)]
    fn connect_unix_async(path: &Path) -> BoxFuture<'_, io::Result<Self::UnixStream>> {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }
}

impl Runtime for Blocking {}

// 's: stream
impl<'s> IoStream<'s, Blocking> for TcpStream {
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = BoxFuture<'s, io::Result<usize>>;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = BoxFuture<'s, io::Result<usize>>;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ShutdownFuture = BoxFuture<'s, io::Result<()>>;

    #[inline]
    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize> {
        Read::read(self, buf)
    }

    #[inline]
    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        let size = buf.len();
        Write::write_all(self, buf)?;

        Ok(size)
    }

    #[inline]
    #[doc(hidden)]
    fn shutdown(&'s mut self) -> io::Result<()> {
        TcpStream::shutdown(self, Shutdown::Both)
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn read_async(&'s mut self, _buf: &'s mut [u8]) -> Self::ReadFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn write_async(&'s mut self, _buf: &'s [u8]) -> Self::WriteFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }
}

// 's: stream
#[cfg(unix)]
impl<'s> IoStream<'s, Blocking> for UnixStream {
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = BoxFuture<'s, io::Result<usize>>;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = BoxFuture<'s, io::Result<usize>>;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ShutdownFuture = BoxFuture<'s, io::Result<()>>;

    #[inline]
    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize> {
        Read::read(self, buf)
    }

    #[inline]
    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        let size = buf.len();
        Write::write_all(self, buf)?;

        Ok(size)
    }

    #[inline]
    #[doc(hidden)]
    fn shutdown(&'s mut self) -> io::Result<()> {
        UnixStream::shutdown(self, Shutdown::Both)
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    #[allow(unused_variables)]
    fn read_async(&'s mut self, _buf: &'s mut [u8]) -> Self::ReadFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    #[allow(unused_variables)]
    fn write_async(&'s mut self, _buf: &'s [u8]) -> Self::WriteFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }
}
