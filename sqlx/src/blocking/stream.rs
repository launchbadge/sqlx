use std::io;

use sqlx_core::io::Stream as IoStream;
use sqlx_core::{Blocking, Runtime};

#[doc(hidden)]
#[allow(clippy::module_name_repetitions)]
pub struct TcpStream(pub(super) <Blocking as Runtime>::TcpStream);

#[doc(hidden)]
#[cfg(unix)]
#[allow(clippy::module_name_repetitions)]
pub struct UnixStream(pub(super) <Blocking as Runtime>::UnixStream);

impl<'s> IoStream<'s, super::Blocking> for TcpStream {
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = <<Blocking as Runtime>::TcpStream as IoStream<'s, Blocking>>::ReadFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = <<Blocking as Runtime>::TcpStream as IoStream<'s, Blocking>>::WriteFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ShutdownFuture =
        <<Blocking as Runtime>::TcpStream as IoStream<'s, Blocking>>::ShutdownFuture;

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

    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[doc(hidden)]
    fn shutdown(&'s mut self) -> io::Result<()> {
        <<Blocking as Runtime>::TcpStream as IoStream<'_, Blocking>>::shutdown(&mut self.0)
    }
}

#[cfg(unix)]
impl<'s> IoStream<'s, super::Blocking> for UnixStream {
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = <<Blocking as Runtime>::UnixStream as IoStream<'s, Blocking>>::ReadFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = <<Blocking as Runtime>::UnixStream as IoStream<'s, Blocking>>::WriteFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ShutdownFuture =
        <<Blocking as Runtime>::UnixStream as IoStream<'s, Blocking>>::ShutdownFuture;

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

    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[doc(hidden)]
    fn shutdown(&'s mut self) -> io::Result<()> {
        <<Blocking as Runtime>::UnixStream as IoStream<'_, Blocking>>::shutdown(&mut self.0)
    }
}
