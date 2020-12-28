#[cfg(feature = "async-std")]
mod async_std;

#[cfg(feature = "actix")]
mod actix;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "async-std")]
pub use self::async_std::AsyncStd;

#[cfg(feature = "tokio")]
pub use self::tokio::Tokio;

#[cfg(feature = "actix")]
pub use self::actix::Actix;

/// Describes a set of types and functions used to open and manage
/// resources within SQLx.
pub trait Runtime {
    type TcpStream;

    /// Opens a TCP connection to a remote host at the specified port.
    #[cfg(feature = "async")]
    #[allow(unused_variables)]
    fn connect_tcp(
        host: &str,
        port: u16,
    ) -> futures_util::future::BoxFuture<'_, std::io::Result<Self::TcpStream>>
    where
        Self: Async,
    {
        unimplemented!()
    }
}

/// Marker trait that identifies a `Runtime` as supporting asynchronous I/O.
#[cfg(feature = "async")]
pub trait Async: Runtime {}
