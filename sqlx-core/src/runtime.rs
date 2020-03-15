#[cfg(feature = "runtime-async-std")]
pub use async_std::{
    fs,
    future::timeout,
    io::prelude::{ReadExt as AsyncReadExt, WriteExt as AsyncWriteExt},
    io::{Read as AsyncRead, Write as AsyncWrite},
    net::TcpStream,
    task::sleep,
    task::spawn,
    task::yield_now,
};

#[cfg(feature = "runtime-tokio")]
pub use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    task::spawn,
    task::yield_now,
    time::delay_for as sleep,
    time::timeout,
};
