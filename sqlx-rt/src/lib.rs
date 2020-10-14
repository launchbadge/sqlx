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
    "only one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features can be enabled"
);

pub use native_tls::{self, Error as TlsError};

//
// Actix *OR* Tokio
//

#[cfg(all(
    any(feature = "runtime-tokio", feature = "runtime-actix"),
    not(feature = "runtime-async-std"),
))]
pub use tokio::{
    self, fs, io::AsyncRead, io::AsyncReadExt, io::AsyncWrite, io::AsyncWriteExt, net::TcpStream,
    task::spawn, task::yield_now, time::delay_for as sleep, time::timeout,
};

#[cfg(all(
    unix,
    any(feature = "runtime-tokio", feature = "runtime-actix"),
    not(feature = "runtime-async-std"),
))]
pub use tokio::net::UnixStream;

#[cfg(all(feature = "tokio-native-tls", not(feature = "async-native-tls")))]
pub use tokio_native_tls::{TlsConnector, TlsStream};

//
// tokio
//

#[cfg(all(
    feature = "runtime-tokio",
    not(any(feature = "runtime-actix", feature = "runtime-async-std",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::tokio::task::block_in_place(move || { $($expr)* })
    };
}

//
// actix
//

#[cfg(feature = "runtime-actix")]
pub use {actix_rt, actix_threadpool};

#[cfg(all(
    feature = "runtime-actix",
    not(any(feature = "runtime-tokio", feature = "runtime-async-std",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::actix_threadpool::run(move || { $($expr)* }).await.map_err(|err| match err {
            $crate::actix_threadpool::BlockingError::Error(e) => e,
            $crate::actix_threadpool::BlockingError::Canceled => panic!("{}", err)
        })
    };
}

//
// async-std
//

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio",))
))]
pub use async_std::{
    self, fs, future::timeout, io::prelude::ReadExt as AsyncReadExt,
    io::prelude::WriteExt as AsyncWriteExt, io::Read as AsyncRead, io::Write as AsyncWrite,
    net::TcpStream, task::sleep, task::spawn, task::yield_now,
};

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::async_std::task::spawn_blocking(move || { $($expr)* }).await
    };
}

#[cfg(all(
    unix,
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio",))
))]
pub use async_std::os::unix::net::UnixStream;

#[cfg(all(feature = "async-native-tls", not(feature = "tokio-native-tls")))]
pub use async_native_tls::{TlsConnector, TlsStream};

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio"))
))]
pub use async_std::task::block_on;

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio"))
))]
pub fn enter_runtime<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // no-op for async-std
    f()
}

#[cfg(all(
    any(feature = "runtime-tokio", feature = "runtime-actix"),
    not(feature = "runtime-async-std")
))]
pub use tokio_runtime::{block_on, enter_runtime};

#[cfg(any(feature = "runtime-tokio", feature = "runtime-actix"))]
mod tokio_runtime {
    use once_cell::sync::Lazy;
    use tokio::runtime::{self, Runtime};

    // lazily initialize a global runtime once for multiple invocations of the macros
    static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
        runtime::Builder::new()
            // `.basic_scheduler()` requires calling `Runtime::block_on()` which needs mutability
            .threaded_scheduler()
            .enable_io()
            .enable_time()
            .build()
            .expect("failed to initialize Tokio runtime")
    });

    pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
        RUNTIME.enter(|| RUNTIME.handle().block_on(future))
    }

    pub fn enter_runtime<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        RUNTIME.enter(f)
    }
}
