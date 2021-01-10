use std::io;

use actix_rt::net::TcpStream;
use async_compat::Compat;
use futures_util::io::{Read, Write};
use futures_util::{future::BoxFuture, AsyncReadExt, AsyncWriteExt, FutureExt, TryFutureExt};

use crate::{io::Stream, Async, Runtime};

/// Actix SQLx runtime. Uses [`actix-rt`][actix_rt] to provide [`Runtime`].
///
/// As of 2021 Jan., Actix re-exports Tokio so this should be equivalent to [`Tokio`][crate::Tokio].
/// This is split-out to allow Actix to shift, or for it to use a different major Tokio version and
/// still work with SQLx.
///
#[cfg_attr(doc_cfg, doc(cfg(feature = "actix")))]
#[derive(Debug)]
pub struct Actix;

impl Runtime for Actix {
    // NOTE: Compat<_> is used to avoid requiring a Box per read/write call
    //       https://github.com/tokio-rs/tokio/issues/2723
    #[doc(hidden)]
    type TcpStream = Compat<TcpStream>;
}

impl Async for Actix {
    #[doc(hidden)]
    fn connect_tcp_async(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).map_ok(Compat::new).boxed()
    }
}

// 's: stream
impl<'s> Stream<'s, Actix> for Compat<TcpStream> {
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
