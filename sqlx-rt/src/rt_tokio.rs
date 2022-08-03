pub use tokio::{
    self, fs, io::AsyncRead, io::AsyncReadExt, io::AsyncWrite, io::AsyncWriteExt, io::ReadBuf,
    net::TcpStream, runtime::Handle, sync::Mutex as AsyncMutex, task::spawn, task::yield_now,
    time::sleep, time::timeout,
};

#[cfg(unix)]
pub use tokio::net::UnixStream;

use once_cell::sync::Lazy;
use tokio::runtime::{self, Runtime};

#[cfg(all(feature = "_tls-native-tls", not(feature = "_tls-rustls")))]
pub use tokio_native_tls::{TlsConnector, TlsStream};

#[cfg(all(feature = "_tls-rustls", not(feature = "_tls-native-tls")))]
pub use tokio_rustls::{client::TlsStream, TlsConnector};

// lazily initialize a global runtime once for multiple invocations of the macros
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("failed to initialize Tokio runtime")
});

pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}

pub fn enter_runtime<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _rt = RUNTIME.enter();
    f()
}

pub fn test_block_on<F: std::future::Future>(future: F) -> F::Output {
    // For tests, we want a single runtime per thread for isolation.
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to initialize Tokio test runtime")
        .block_on(future)
}
