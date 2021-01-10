use std::io;

use _tokio::net::TcpStream;
use async_compat::Compat;
use futures_util::io::{Read, Write};
use futures_util::{future::BoxFuture, AsyncReadExt, AsyncWriteExt, FutureExt, TryFutureExt};

use crate::{io::Stream, Async, Runtime};

/// Provides [`Runtime`] for [**Tokio**](https://tokio.rs). Supports only non-blocking operation.
///
/// SQLx does not require the use of a multi-threaded executor.
///
#[cfg_attr(doc_cfg, doc(cfg(feature = "tokio")))]
#[derive(Debug)]
pub struct Tokio;

impl Runtime for Tokio {
    // NOTE: Compat<_> is used to avoid requiring a Box per read/write call
    //       https://github.com/tokio-rs/tokio/issues/2723
    #[doc(hidden)]
    type TcpStream = Compat<TcpStream>;
}

impl Async for Tokio {
    #[doc(hidden)]
    fn connect_tcp_async(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).map_ok(Compat::new).boxed()
    }
}

// 's: stream
impl<'s> Stream<'s, Tokio> for Compat<TcpStream> {
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
