use std::io;
#[cfg(unix)]
use std::path::Path;

use actix_rt::net::TcpStream;
#[cfg(unix)]
use actix_rt::net::UnixStream;
use async_compat::Compat;
use futures_util::io::{Read, Write};
use futures_util::{future::BoxFuture, AsyncReadExt, AsyncWriteExt, FutureExt, TryFutureExt};

use crate::{io::Stream, Async, Runtime};

/// Provides [`Runtime`] for [**Actix**](https://actix.rs). Supports only non-blocking operation.
///
/// As of 2021 Jan., Actix re-exports Tokio so this should be equivalent to [`Tokio`][crate::Tokio].
/// This is split-out to allow Actix to shift, or for it to use a different major Tokio version and
/// still work with SQLx.
///
#[cfg_attr(doc_cfg, doc(cfg(feature = "actix")))]
#[derive(Debug)]
pub struct Actix;

// NOTE: Compat<_> is used for IO streams to avoid requiring a Box per read/write call
//       https://github.com/tokio-rs/tokio/issues/2723
impl Runtime for Actix {
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
}

impl Async for Actix {}

// 's: stream
impl<'s> Stream<'s, Actix> for Compat<TcpStream> {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[inline]
    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture {
        AsyncReadExt::read(self, buf)
    }

    #[inline]
    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        AsyncWriteExt::write(self, buf)
    }

    #[inline]
    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, _buf: &'s mut [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[inline]
    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, _buf: &'s [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }
}

// 's: stream
#[cfg(unix)]
impl<'s> Stream<'s, Actix> for Compat<UnixStream> {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[inline]
    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture {
        AsyncReadExt::read(self, buf)
    }

    #[inline]
    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        AsyncWriteExt::write(self, buf)
    }

    #[inline]
    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, _buf: &'s mut [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }

    #[inline]
    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, _buf: &'s [u8]) -> io::Result<usize> {
        // UNREACHABLE: where Self: blocking::Runtime
        unreachable!()
    }
}
