use std::io;

use futures_util::future::BoxFuture;

#[cfg(feature = "async-std")]
pub(crate) mod async_std;

#[cfg(feature = "actix")]
pub(crate) mod actix;

#[cfg(feature = "tokio")]
pub(crate) mod tokio;

/// Describes a set of types and functions used to open and manage
/// resources within SQLx using asynchronous I/O.
#[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "async-std", feature = "tokio", feature = "actix")))
)]
pub trait Runtime {
    type TcpStream;

    /// Opens a TCP connection to a remote host at the specified port.
    fn connect_tcp(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>>;
}
