use std::io;

use actix_rt::net::TcpStream;
use async_compat_02::Compat;
use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};

use crate::{AsyncRuntime, Runtime};

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
    type TcpStream = Compat<TcpStream>;
}

impl AsyncRuntime for Actix
where
    Self::TcpStream: futures_io::AsyncRead,
{
    fn connect_tcp(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).map_ok(Compat::new).boxed()
    }
}
