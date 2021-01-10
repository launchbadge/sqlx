use _async_std::net::TcpStream;
#[cfg(feature = "blocking")]
use _async_std::task;
use futures_util::io::{Read, Write};
use futures_util::{future::BoxFuture, AsyncReadExt, AsyncWriteExt, FutureExt};

#[cfg(feature = "blocking")]
use crate::blocking;
use crate::{io::Stream, Async, Runtime};

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
}

impl Async for AsyncStd {
    #[doc(hidden)]
    fn connect_tcp_async(host: &str, port: u16) -> BoxFuture<'_, std::io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).boxed()
    }
}

#[cfg(feature = "blocking")]
impl blocking::Runtime for AsyncStd {
    #[doc(hidden)]
    fn connect_tcp(host: &str, port: u16) -> std::io::Result<Self::TcpStream> {
        task::block_on(Self::connect_tcp_async(host, port))
    }
}

// 's: stream
impl<'s> Stream<'s, AsyncStd> for TcpStream {
    #[doc(hidden)]
    type ReadFuture = Read<'s, Self>;

    #[doc(hidden)]
    type WriteFuture = Write<'s, Self>;

    #[inline]
    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture {
        self.read(buf)
    }

    #[inline]
    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        self.write(buf)
    }
}

// 's: stream
#[cfg(feature = "blocking")]
impl<'s> blocking::io::Stream<'s, AsyncStd> for TcpStream {
    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> std::io::Result<usize> {
        _async_std::task::block_on(self.read_async(buf))
    }

    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> std::io::Result<usize> {
        _async_std::task::block_on(self.write_async(buf))
    }
}
