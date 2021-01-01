use std::io;

use async_compat::Compat;
use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};
use tokio::net::TcpStream;

use crate::{AsyncRuntime, Runtime};

/// Tokio SQLx runtime. Uses [`tokio`] to provide [`Runtime`].
///
/// SQLx does not require the use of a multi-threaded executor.
///
#[cfg_attr(doc_cfg, doc(cfg(feature = "tokio")))]
#[derive(Debug)]
pub struct Tokio;

impl Runtime for Tokio {
    type TcpStream = Compat<TcpStream>;
}

impl AsyncRuntime for Tokio {
    fn connect_tcp(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).map_ok(Compat::new).boxed()
    }
}
