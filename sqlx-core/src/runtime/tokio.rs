use std::io;
#[cfg(unix)]
use std::path::Path;

use _tokio::net::TcpStream;
#[cfg(unix)]
use _tokio::net::UnixStream;
use async_compat::Compat;
use futures_util::future::BoxFuture;
use futures_util::io::{Read, Write};
use futures_util::{AsyncReadExt, AsyncWriteExt, FutureExt, TryFutureExt};

use crate::io::Stream;
use crate::{Async, Runtime};
use std::future::Future;
use std::time::{Duration, Instant};

/// Provides [`Runtime`] for [**Tokio**](https://tokio.rs). Supports only non-blocking operation.
///
/// SQLx does not require the use of a multi-threaded executor.
///
#[cfg_attr(doc_cfg, doc(cfg(feature = "tokio")))]
#[derive(Debug)]
pub struct Tokio;

// NOTE: Compat<_> is used for IO streams to avoid requiring a Box per read/write call
//       https://github.com/tokio-rs/tokio/issues/2723
impl Runtime for Tokio {
    #[doc(hidden)]
    type TcpStream = Compat<TcpStream>;

    #[doc(hidden)]
    #[cfg(unix)]
    type UnixStream = Compat<UnixStream>;

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn connect_tcp(_host: &str, _port: u16) -> io::Result<Self::TcpStream> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(all(unix, feature = "blocking"))]
    fn connect_unix(_path: &Path) -> io::Result<Self::UnixStream> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[doc(hidden)]
    fn connect_tcp_async(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).map_ok(Compat::new).boxed()
    }

    #[doc(hidden)]
    #[cfg(unix)]
    fn connect_unix_async(path: &Path) -> BoxFuture<'_, io::Result<Self::UnixStream>> {
        UnixStream::connect(path).map_ok(Compat::new).boxed()
    }

    #[doc(hidden)]
    fn timeout_async<'a, F: Future + Send + 'a>(
        fut: F,
        timeout: Duration,
    ) -> BoxFuture<'a, Option<F::Output>> {
        Box::pin(_tokio::time::timeout(timeout.into(), fut).map(Result::ok))
    }
}

impl Async for Tokio {}

// 's: stream
impl<'s> Stream<'s, Tokio> for Compat<TcpStream> {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[doc(hidden)]
    type ShutdownFuture = BoxFuture<'s, io::Result<()>>;

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
        _tokio::io::AsyncWriteExt::shutdown(self.get_mut()).boxed()
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, _buf: &'s mut [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, _buf: &'s [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<()> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }
}

// 's: stream
#[cfg(unix)]
impl<'s> Stream<'s, Tokio> for Compat<UnixStream> {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[doc(hidden)]
    type ShutdownFuture = BoxFuture<'s, io::Result<()>>;

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
        _tokio::io::AsyncWriteExt::shutdown(self.get_mut()).boxed()
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, _buf: &'s mut [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, _buf: &'s [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<()> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }
}
