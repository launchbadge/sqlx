use std::io;
#[cfg(unix)]
use std::path::Path;

#[cfg(feature = "async")]
use futures_util::future::BoxFuture;
use sqlx_core::Runtime;

use super::stream::{TcpStream, UnixStream};

/// Uses the `std::net` primitives to implement a blocking runtime for SQLx.
#[derive(Debug)]
pub struct Blocking(sqlx_core::Blocking);

impl Runtime for Blocking {
    #[doc(hidden)]
    type TcpStream = TcpStream;

    #[doc(hidden)]
    #[cfg(unix)]
    type UnixStream = UnixStream;

    #[doc(hidden)]
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream>
    where
        Self: sqlx_core::blocking::Runtime,
    {
        sqlx_core::Blocking::connect_tcp(host, port).map(TcpStream)
    }

    #[doc(hidden)]
    #[cfg(unix)]
    fn connect_unix(path: &Path) -> io::Result<Self::UnixStream>
    where
        Self: sqlx_core::blocking::Runtime,
    {
        sqlx_core::Blocking::connect_unix(path).map(UnixStream)
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn connect_tcp_async(_host: &str, _port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn connect_unix_async(_path: &Path) -> BoxFuture<'_, io::Result<Self::UnixStream>> {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }
}

impl sqlx_core::blocking::Runtime for Blocking {}
