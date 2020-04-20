#[cfg(not(any(
    feature = "runtime-actix",
    feature = "runtime-async-std",
    feature = "runtime-tokio",
)))]
compile_error!(
    "one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features must be enabled"
);

#[cfg(any(
    all(feature = "runtime-actix", feature = "runtime-async-std"),
    all(feature = "runtime-actix", feature = "runtime-tokio"),
    all(feature = "runtime-async-std", feature = "runtime-tokio"),
))]
compile_error!(
    "only one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features must be enabled"
);

pub use sqlx_rt_macros::{main, test};

pub use native_tls;

#[cfg(any(feature = "runtime-tokio", feature = "runtime-actix"))]
pub use tokio::{
    self, fs, io::AsyncRead, io::AsyncReadExt, io::AsyncWrite, io::AsyncWriteExt, net::TcpStream,
    task::yield_now,
};

#[cfg(feature = "runtime-tokio")]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::tokio::task::block_in_place(move || { $($expr)* })
    };
}

#[cfg(feature = "runtime-actix")]
pub use {actix_rt, actix_threadpool};

#[cfg(feature = "runtime-actix")]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::actix_threadpool::run(move || { $($expr)* }).await.map_err(|err| match err {
            $crate::actix_threadpool::BlockingError::Error(e) => e,
            $crate::actix_threadpool::BlockingError::Canceled => panic!("{}", err)
        })
    };
}

#[cfg(all(unix, any(feature = "runtime-tokio", feature = "runtime-actix")))]
pub use tokio::net::UnixStream;

#[cfg(feature = "runtime-async-std")]
pub use async_std::{
    self, fs, io::prelude::ReadExt as AsyncReadExt, io::prelude::WriteExt as AsyncWriteExt,
    io::Read as AsyncRead, io::Write as AsyncWrite, net::TcpStream, task::spawn, task::yield_now,
};

#[cfg(feature = "runtime-async-std")]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::async_std::task::spawn_blocking(move || { $($expr)* }).await
    };
}

#[cfg(all(unix, feature = "runtime-async-std"))]
pub use async_std::os::unix::net::UnixStream;

#[cfg(feature = "async-native-tls")]
pub use async_native_tls::{Error as TlsError, TlsConnector, TlsStream};

#[cfg(feature = "tokio-native-tls")]
pub use tokio_native_tls::{TlsConnector, TlsStream};

#[cfg(feature = "tokio-native-tls")]
pub use native_tls::Error as TlsError;
