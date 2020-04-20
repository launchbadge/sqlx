use std::io;
use std::net::Shutdown;
use std::pin::Pin;
use std::task::{Context, Poll};

use sqlx_rt::{AsyncRead, AsyncWrite, TcpStream};

#[derive(Debug)]
pub enum Socket {
    Tcp(TcpStream),

    #[cfg(unix)]
    Unix(sqlx_rt::UnixStream),
}

impl Socket {
    #[cfg(not(unix))]
    pub async fn connect(host: &str, port: u16) -> io::Result<Self> {
        TcpStream::connect((host, port)).await.map(Socket::Tcp)
    }

    #[cfg(unix)]
    pub async fn connect(host: &str, port: u16) -> io::Result<Self> {
        if host.starts_with('/') {
            // if the host starts with a forward slash, assume that this is a request
            // to connect to a local socket
            sqlx_rt::UnixStream::connect(format!("{}/.s.PGSQL.{}", host, port))
                .await
                .map(Socket::Unix)
        } else {
            TcpStream::connect((host, port)).await.map(Socket::Tcp)
        }
    }

    pub fn shutdown(&self) -> io::Result<()> {
        match self {
            Socket::Tcp(s) => s.shutdown(Shutdown::Both),

            #[cfg(unix)]
            Socket::Unix(s) => s.shutdown(Shutdown::Both),
        }
    }
}

impl AsyncRead for Socket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_read(cx, buf),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_read(cx, buf),
        }
    }

    #[cfg(any(feature = "runtime-actix", feature = "runtime-tokio"))]
    fn poll_read_buf<B>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>>
    where
        Self: Sized,
        B: bytes::BufMut,
    {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_read_buf(cx, buf),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_read_buf(cx, buf),
        }
    }
}

impl AsyncWrite for Socket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_write(cx, buf),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_flush(cx),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_flush(cx),
        }
    }

    #[cfg(any(feature = "runtime-actix", feature = "runtime-tokio"))]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_shutdown(cx),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_shutdown(cx),
        }
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_close(cx),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_close(cx),
        }
    }

    #[cfg(any(feature = "runtime-actix", feature = "runtime-tokio"))]
    fn poll_write_buf<B>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>>
    where
        Self: Sized,
        B: bytes::Buf,
    {
        match &mut *self {
            Socket::Tcp(s) => Pin::new(s).poll_write_buf(cx, buf),

            #[cfg(unix)]
            Socket::Unix(s) => Pin::new(s).poll_write_buf(cx, buf),
        }
    }
}
