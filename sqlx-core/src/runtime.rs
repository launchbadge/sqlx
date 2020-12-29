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
pub trait Runtime: 'static + Send + Sync {
    type TcpStream: Send;

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
        // re-implemented for async runtimes
        // for sync runtimes, this cannot be implemented but the compiler
        // with guarantee it won't be called
        // see: https://github.com/rust-lang/rust/issues/48214
        unimplemented!()
    }
}

/// Marker trait that identifies a `Runtime` as supporting asynchronous I/O.
#[cfg(feature = "async")]
pub trait Async: Runtime {}

// when the async feature is not specified, this is an empty trait
// we implement `()` for it to allow the lib to still compile
#[cfg(not(feature = "async"))]
impl Runtime for () {
    type TcpStream = ();
}
