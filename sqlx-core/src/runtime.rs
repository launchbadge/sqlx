#[cfg(feature = "runtime-async-std")]
pub use async_std::{
    fs,
    future::timeout,
    io::prelude::{ReadExt as AsyncReadExt, WriteExt as AsyncWriteExt},
    io::{Read as AsyncRead, Write as AsyncWrite},
    net::TcpStream,
    task::sleep,
    task::yield_now,
    task::{spawn, spawn_blocking},
};

#[cfg(feature = "runtime-tokio")]
pub use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    task::yield_now,
    task::{spawn, spawn_blocking},
    time::delay_for as sleep,
    time::timeout,
};
