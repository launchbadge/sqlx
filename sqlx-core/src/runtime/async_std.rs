use std::io;
use std::net::Shutdown;
#[cfg(unix)]
use std::path::Path;

use _async_std::net::TcpStream;
#[cfg(unix)]
use _async_std::os::unix::net::UnixStream;
#[cfg(feature = "blocking")]
use _async_std::task;
use futures_util::future::{self, BoxFuture};
use futures_util::io::{Read, Write};
use futures_util::{AsyncReadExt, AsyncWriteExt, FutureExt};

#[cfg(feature = "blocking")]
use crate::blocking;
use crate::io::Stream;
use crate::{Async, Runtime};

/// Provides [`Runtime`] for [**async-std**](https://async.rs). Supports both blocking
/// and non-blocking operation.
///
/// For blocking operation, the equivalent non-blocking methods are called
/// and trivially wrapped in [`task::block_on`][task::block_on].
///
#[cfg_attr(doc_cfg, doc(cfg(feature = "async-std")))]
#[derive(Debug)]
pub struct AsyncStd;

impl Runtime for AsyncStd {
    #[doc(hidden)]
    type TcpStream = TcpStream;

    #[doc(hidden)]
    #[cfg(unix)]
    type UnixStream = UnixStream;

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream> {
        task::block_on(Self::connect_tcp_async(host, port))
    }

    #[doc(hidden)]
    #[cfg(all(unix, feature = "blocking"))]
    fn connect_unix(path: &Path) -> io::Result<Self::UnixStream> {
        task::block_on(Self::connect_unix_async(path))
    }

    #[doc(hidden)]
    fn connect_tcp_async(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).boxed()
    }

    #[doc(hidden)]
    #[cfg(unix)]
    fn connect_unix_async(path: &Path) -> BoxFuture<'_, io::Result<Self::UnixStream>> {
        UnixStream::connect(path).boxed()
    }
}

impl Async for AsyncStd {}

// blocking operations provided by trivially wrapping async counterparts
// with `task::block_on`
#[cfg(feature = "blocking")]
impl blocking::Runtime for AsyncStd {}

// 's: stream
impl<'s> Stream<'s, AsyncStd> for TcpStream {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[doc(hidden)]
    type ShutdownFuture = future::Ready<io::Result<()>>;

    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture {
        AsyncReadExt::read(self, buf)
    }

    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        AsyncWriteExt::write(self, buf)
    }

    #[doc(hidden)]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture {
        future::ready(Self::shutdown(self, Shutdown::Both))
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize> {
        task::block_on(self.read_async(buf))
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        task::block_on(self.write_async(buf))
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<()> {
        task::block_on(self.shutdown_async())
    }
}

// 's: stream
#[cfg(unix)]
impl<'s> Stream<'s, AsyncStd> for UnixStream {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[doc(hidden)]
    type ShutdownFuture = future::Ready<io::Result<()>>;

    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture {
        AsyncReadExt::read(self, buf)
    }

    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        AsyncWriteExt::write(self, buf)
    }

    #[doc(hidden)]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture {
        future::ready(Self::shutdown(self, Shutdown::Both))
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize> {
        task::block_on(self.read_async(buf))
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        task::block_on(self.write_async(buf))
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<()> {
        task::block_on(self.shutdown_async())
    }
}
