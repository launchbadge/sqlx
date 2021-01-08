#[cfg(feature = "_mock")]
#[doc(hidden)]
pub mod mock;

#[cfg(feature = "async-std")]
#[path = "runtime/async_std.rs"]
mod async_std_;

#[cfg(feature = "actix")]
#[path = "runtime/actix.rs"]
mod actix_;

#[cfg(feature = "tokio")]
#[path = "runtime/tokio.rs"]
mod tokio_;

#[cfg(feature = "actix")]
pub use actix_::Actix;
#[cfg(feature = "async-std")]
pub use async_std_::AsyncStd;
#[cfg(feature = "_mock")]
pub use mock::Mock;
#[cfg(feature = "tokio")]
pub use tokio_::Tokio;

/// Describes a set of types and functions used to open and manage
/// resources within SQLx.
pub trait Runtime: 'static + Send + Sync {
    type TcpStream: Send;
}

#[cfg(feature = "async")]
pub trait AsyncRuntime: Runtime {
    /// Opens a TCP connection to a remote host at the specified port.
    fn connect_tcp(
        host: &str,
        port: u16,
    ) -> futures_util::future::BoxFuture<'_, std::io::Result<Self::TcpStream>>;
}

// when the async feature is not specified, this is an empty trait
// we implement `()` for it to allow the lib to still compile
#[cfg(not(any(
    feature = "async_std",
    feature = "actix",
    feature = "tokio",
    feature = "blocking"
)))]
impl Runtime for () {
    type TcpStream = ();
}
