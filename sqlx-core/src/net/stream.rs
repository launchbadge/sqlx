use std::io;
use std::path::PathBuf;

use either::Either;
#[cfg(feature = "async")]
use futures_util::future::{self, FutureExt};

use crate::io::Stream as IoStream;
use crate::Runtime;

#[derive(Debug)]
pub enum Stream<Rt>
where
    Rt: Runtime,
{
    Tcp(Rt::TcpStream),

    #[cfg(unix)]
    Unix(Rt::UnixStream),
}

impl<Rt> Stream<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    pub async fn connect_async(address: Either<&(String, u16), &PathBuf>) -> io::Result<Self>
    where
        Rt: crate::Async,
    {
        match address {
            Either::Left((host, port)) => Rt::connect_tcp_async(host, *port).await.map(Self::Tcp),

            #[cfg(unix)]
            Either::Right(socket) => Rt::connect_unix_async(socket).await.map(Self::Unix),

            #[cfg(not(unix))]
            Either(_socket) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unix streams are not supported outside Unix platforms",
            )),
        }
    }

    #[cfg(feature = "blocking")]
    pub fn connect(address: Either<&(String, u16), &PathBuf>) -> io::Result<Self>
    where
        Rt: crate::blocking::Runtime,
    {
        match address {
            Either::Left((host, port)) => Rt::connect_tcp(host, *port).map(Self::Tcp),

            #[cfg(unix)]
            Either::Right(socket) => Rt::connect_unix(socket).map(Self::Unix),

            #[cfg(not(unix))]
            Either(_socket) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unix streams are not supported outside Unix platforms",
            )),
        }
    }
}

#[cfg(unix)]
impl<'s, Rt> IoStream<'s, Rt> for Stream<Rt>
where
    Rt: Runtime,
{
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = future::Either<
        <Rt::TcpStream as IoStream<'s, Rt>>::ReadFuture,
        <Rt::UnixStream as IoStream<'s, Rt>>::ReadFuture,
    >;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = future::Either<
        <Rt::TcpStream as IoStream<'s, Rt>>::WriteFuture,
        <Rt::UnixStream as IoStream<'s, Rt>>::WriteFuture,
    >;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ShutdownFuture = future::Either<
        <Rt::TcpStream as IoStream<'s, Rt>>::ShutdownFuture,
        <Rt::UnixStream as IoStream<'s, Rt>>::ShutdownFuture,
    >;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture
    where
        Rt: crate::Async,
    {
        match self {
            Self::Tcp(stream) => stream.read_async(buf).left_future(),
            Self::Unix(stream) => stream.read_async(buf).right_future(),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture
    where
        Rt: crate::Async,
    {
        match self {
            Self::Tcp(stream) => stream.write_async(buf).left_future(),
            Self::Unix(stream) => stream.write_async(buf).right_future(),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture
    where
        Rt: crate::Async,
    {
        match self {
            Self::Tcp(stream) => stream.shutdown_async().left_future(),
            Self::Unix(stream) => stream.shutdown_async().right_future(),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime,
    {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            Self::Unix(stream) => stream.read(buf),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime,
    {
        match self {
            Self::Tcp(stream) => stream.write(buf),
            Self::Unix(stream) => stream.write(buf),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<()>
    where
        Rt: crate::blocking::Runtime,
    {
        match self {
            Self::Tcp(stream) => stream.shutdown(),
            Self::Unix(stream) => stream.shutdown(),
        }
    }
}

#[cfg(not(unix))]
impl<'s, Rt> IoStream<'s, Rt> for Stream<Rt>
where
    Rt: Runtime,
{
    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ReadFuture = <Rt::TcpStream as IoStream<'s, Rt>>::ReadFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type WriteFuture = <Rt::TcpStream as IoStream<'s, Rt>>::WriteFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    type ShutdownFuture = <Rt::TcpStream as IoStream<'s, Rt>>::ShutdownFuture;

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture
    where
        Rt: crate::Async,
    {
        match self {
            Self::Tcp(stream) => stream.read_async(buf),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture
    where
        Rt: crate::Async,
    {
        match self {
            Self::Tcp(stream) => stream.write_async(buf),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture
    where
        Rt: crate::Async,
    {
        match self {
            Self::Tcp(stream) => stream.shutdown_async().left_future(),
            Self::Unix(stream) => stream.shutdown_async().right_future(),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime,
    {
        match self {
            Self::Tcp(stream) => stream.read(buf),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime,
    {
        match self {
            Self::Tcp(stream) => stream.write(buf),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime,
    {
        match self {
            Self::Tcp(stream) => stream.shutdown(buf),
        }
    }
}
