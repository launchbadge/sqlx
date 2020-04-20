//
// We would like to see a generalized runtime type parameter that is defaulted on types.
// Something like:
//
// ```rust
// PgPool::new("postgres://...") // defaults to async-std
// PgPool::new_with("postgres://...", Tokio) // here is Tokio
// ```
//
// We also do not have the time to invest in bringing the community together on that. We have
// confidence that _something_ will emerge over the year. When such a time comes, the following
// egregious hack should be ruthlessly purged.
//

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

#[cfg(feature = "runtime-async-std")]
pub(crate) use async_std::{
    self, fs,
    future::timeout,
    io::prelude::ReadExt as AsyncReadExt,
    io::prelude::WriteExt as AsyncWriteExt,
    io::{Read as AsyncRead, Write as AsyncWrite},
    net::TcpStream,
    task::sleep,
    task::spawn,
};

#[cfg(all(feature = "runtime-async-std", unix))]
pub(crate) use async_std::os::unix::net::UnixStream;

#[cfg(any(feature = "runtime-tokio", feature = "runtime-actix"))]
pub(crate) use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    time::delay_for as sleep,
    time::timeout,
};

#[cfg(feature = "runtime-tokio")]
pub use tokio::task::spawn;

#[cfg(feature = "runtime-actix")]
pub use actix_rt::spawn;

#[cfg(all(any(feature = "runtime-tokio", feature = "runtime-actix"), unix))]
pub use tokio::net::UnixStream;
