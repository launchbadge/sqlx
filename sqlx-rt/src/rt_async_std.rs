pub use async_std::{
    self, fs, future::timeout, io::prelude::ReadExt as AsyncReadExt,
    io::prelude::WriteExt as AsyncWriteExt, io::Read as AsyncRead, io::Write as AsyncWrite,
    net::TcpStream, sync::Mutex as AsyncMutex, task::sleep, task::spawn, task::yield_now,
};

#[cfg(unix)]
pub use async_std::os::unix::net::UnixStream;

#[cfg(all(feature = "_tls-native-tls", not(feature = "_tls-rustls")))]
pub use async_native_tls::{TlsConnector, TlsStream};

#[cfg(all(feature = "_tls-rustls", not(feature = "_tls-native-tls")))]
pub use futures_rustls::{client::TlsStream, TlsConnector};

pub use async_std::task::{block_on, block_on as test_block_on};

pub fn enter_runtime<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // no-op for async-std
    f()
}
