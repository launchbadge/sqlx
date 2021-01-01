#[cfg(feature = "async-std")]
mod async_std;

#[cfg(feature = "actix")]
mod actix;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "actix")]
pub use self::actix::Actix;
#[cfg(feature = "async-std")]
pub use self::async_std::AsyncStd;
#[cfg(feature = "tokio")]
pub use self::tokio::Tokio;

/// Describes a set of types and functions used to open and manage
/// resources within SQLx.
pub trait Runtime: 'static + Send + Sync {
    type TcpStream: Send;
}

#[cfg(feature = "async")]
pub trait AsyncRuntime: Runtime
where
    Self::TcpStream: futures_io::AsyncRead,
{
    /// Opens a TCP connection to a remote host at the specified port.
    fn connect_tcp(
        host: &str,
        port: u16,
    ) -> futures_util::future::BoxFuture<'_, std::io::Result<Self::TcpStream>>;
}

#[cfg(feature = "async")]
pub trait AsyncRead {
    fn read(&mut self, buf: &mut [u8]) -> futures_util::future::BoxFuture<'_, u64>;
}

// when the async feature is not specified, this is an empty trait
// we implement `()` for it to allow the lib to still compile
#[cfg(not(feature = "async"))]
impl Runtime for () {
    type TcpStream = ();
}
