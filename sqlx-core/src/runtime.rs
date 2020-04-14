#![allow(unused_imports)]

#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(feature = "runtime-async-std")]
pub(crate) use async_std::{
    fs,
    future::timeout,
    io::prelude::ReadExt as AsyncReadExt,
    io::{Read as AsyncRead, Write as AsyncWrite},
    net::TcpStream,
    task::sleep,
    task::spawn,
};

#[cfg(all(feature = "runtime-async-std", feature = "postgres", unix))]
pub(crate) use async_std::os::unix::net::UnixStream;

#[cfg(feature = "runtime-tokio")]
pub(crate) use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    net::TcpStream,
    task::spawn,
    time::delay_for as sleep,
    time::timeout,
};

#[cfg(all(feature = "runtime-tokio", feature = "postgres", unix))]
pub(crate) use tokio::net::UnixStream;
