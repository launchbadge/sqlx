use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::BufMut;
use futures_core::ready;

pub use buffered::{BufferedSocket, WriteBuffer};

use crate::io::ReadBuf;

mod buffered;

pub trait Socket: Send + Sync + Unpin + 'static {
    fn try_read(&mut self, buf: &mut dyn ReadBuf) -> io::Result<usize>;

    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize>;

    fn poll_read_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>>;

    fn poll_write_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>>;

    fn poll_flush(&mut self, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // `flush()` is a no-op for TCP/UDS
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>>;

    fn read<'a, B: ReadBuf>(&'a mut self, buf: &'a mut B) -> Read<'a, Self, B>
    where
        Self: Sized,
    {
        Read { socket: self, buf }
    }

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Write<'a, Self>
    where
        Self: Sized,
    {
        Write { socket: self, buf }
    }

    fn flush(&mut self) -> Flush<'_, Self>
    where
        Self: Sized,
    {
        Flush { socket: self }
    }

    fn shutdown(&mut self) -> Shutdown<'_, Self>
    where
        Self: Sized,
    {
        Shutdown { socket: self }
    }
}

pub struct Read<'a, S: ?Sized, B> {
    socket: &'a mut S,
    buf: &'a mut B,
}

impl<'a, S: ?Sized, B> Future for Read<'a, S, B>
where
    S: Socket,
    B: ReadBuf,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        while this.buf.has_remaining_mut() {
            match this.socket.try_read(&mut *this.buf) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    ready!(this.socket.poll_read_ready(cx))?;
                }
                ready => return Poll::Ready(ready),
            }
        }

        Poll::Ready(Ok(0))
    }
}

pub struct Write<'a, S: ?Sized> {
    socket: &'a mut S,
    buf: &'a [u8],
}

impl<'a, S: ?Sized> Future for Write<'a, S>
where
    S: Socket,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        while !this.buf.is_empty() {
            match this.socket.try_write(this.buf) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    ready!(this.socket.poll_write_ready(cx))?;
                }
                ready => return Poll::Ready(ready),
            }
        }

        Poll::Ready(Ok(0))
    }
}

pub struct Flush<'a, S: ?Sized> {
    socket: &'a mut S,
}

impl<'a, S: Socket + ?Sized> Future for Flush<'a, S> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.socket.poll_flush(cx)
    }
}

pub struct Shutdown<'a, S: ?Sized> {
    socket: &'a mut S,
}

impl<'a, S: ?Sized> Future for Shutdown<'a, S>
where
    S: Socket,
{
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.socket.poll_shutdown(cx)
    }
}

pub trait WithSocket {
    type Output;

    fn with_socket<S: Socket>(self, socket: S) -> Self::Output;
}

pub struct SocketIntoBox;

impl WithSocket for SocketIntoBox {
    type Output = Box<dyn Socket>;

    fn with_socket<S: Socket>(self, socket: S) -> Self::Output {
        Box::new(socket)
    }
}

impl<S: Socket + ?Sized> Socket for Box<S> {
    fn try_read(&mut self, buf: &mut dyn ReadBuf) -> io::Result<usize> {
        (**self).try_read(buf)
    }

    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).try_write(buf)
    }

    fn poll_read_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        (**self).poll_read_ready(cx)
    }

    fn poll_write_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        (**self).poll_write_ready(cx)
    }

    fn poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        (**self).poll_flush(cx)
    }

    fn poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        (**self).poll_shutdown(cx)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TcpKeepalive {
    pub time: Option<Duration>,
    pub interval: Option<Duration>,
    pub retries: Option<u32>,
}

impl TcpKeepalive {
    /// Returns a new, empty set of TCP keepalive parameters.
    pub const fn new() -> TcpKeepalive {
        TcpKeepalive {
            time: None,
            interval: None,
            retries: None,
        }
    }

    /// Set the amount of time after which TCP keepalive probes will be sent on
    /// idle connections.
    ///
    /// This will set `TCP_KEEPALIVE` on macOS and iOS, and
    /// `TCP_KEEPIDLE` on all other Unix operating systems, except
    /// OpenBSD and Haiku which don't support any way to set this
    /// option. On Windows, this sets the value of the `tcp_keepalive`
    /// struct's `keepalivetime` field.
    ///
    /// Some platforms specify this value in seconds, so sub-second
    /// specifications may be omitted.
    pub const fn with_time(self, time: Duration) -> Self {
        Self {
            time: Some(time),
            ..self
        }
    }

    /// Set the value of the `TCP_KEEPINTVL` option. On Windows, this sets the
    /// value of the `tcp_keepalive` struct's `keepaliveinterval` field.
    ///
    /// Sets the time interval between TCP keepalive probes.
    ///
    /// Some platforms specify this value in seconds, so sub-second
    /// specifications may be omitted.
    pub const fn with_interval(self, interval: Duration) -> Self {
        Self {
            interval: Some(interval),
            ..self
        }
    }

    /// Set the value of the `TCP_KEEPCNT` option.
    ///
    /// Set the maximum number of TCP keepalive probes that will be sent before
    /// dropping a connection, if TCP keepalive is enabled on this socket.
    pub const fn with_retries(self, retries: u32) -> Self {
        Self {
            retries: Some(retries),
            ..self
        }
    }

    /// Convert `TcpKeepalive` to `socket2::TcpKeepalive`.
    pub const fn socket2(self) -> socket2::TcpKeepalive {
        let mut ka = socket2::TcpKeepalive::new();
        if let Some(time) = self.time {
            ka = ka.with_time(time);
        }
        if let Some(interval) = self.interval {
            ka = ka.with_interval(interval);
        }
        if let Some(retries) = self.retries {
            ka = ka.with_retries(retries);
        }
        ka
    }
}

pub async fn connect_tcp<Ws: WithSocket>(
    host: &str,
    port: u16,
    with_socket: Ws,
    keepalive: Option<&TcpKeepalive>,
) -> crate::Result<Ws::Output> {
    // IPv6 addresses in URLs will be wrapped in brackets and the `url` crate doesn't trim those.
    let host = host.trim_matches(&['[', ']'][..]);

    #[cfg(feature = "_rt-tokio")]
    if crate::rt::rt_tokio::available() {
        use tokio::net::TcpStream;

        let stream = TcpStream::connect((host, port)).await?;
        stream.set_nodelay(true)?;

        // set tcp keepalive
        if let Some(keepalive) = keepalive {
            let keepalive = keepalive.socket2();
            let sock_ref = socket2::SockRef::from(&stream);
            sock_ref.set_tcp_keepalive(&keepalive)?;
        }

        return Ok(with_socket.with_socket(stream));
    }

    #[cfg(feature = "_rt-async-std")]
    {
        use async_io::Async;
        use async_std::net::ToSocketAddrs;
        use std::net::TcpStream;

        let mut last_err = None;

        // Loop through all the Socket Addresses that the hostname resolves to
        for socket_addr in (host, port).to_socket_addrs().await? {
            let stream = Async::<TcpStream>::connect(socket_addr)
                .await
                .and_then(|s| {
                    s.get_ref().set_nodelay(true)?;
                    Ok(s)
                });
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            };
            // set tcp keepalive
            if let Some(keepalive) = keepalive {
                let keepalive = socket2::TcpKeepalive::new()
                    .with_interval(keepalive.interval)
                    .with_retries(keepalive.retries)
                    .with_time(keepalive.time);
                let sock_ref = socket2::SockRef::from(&stream);
                match sock_ref.set_tcp_keepalive(&keepalive) {
                    Ok(_) => return Ok(with_socket.with_socket(stream)),
                    Err(e) => last_err = Some(e),
                }
            }
        }

        // If we reach this point, it means we failed to connect to any of the addresses.
        // Return the last error we encountered, or a custom error if the hostname didn't resolve to any address.
        match last_err {
            Some(err) => Err(err.into()),
            None => Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "Hostname did not resolve to any addresses",
            )
            .into()),
        }
    }

    #[cfg(not(feature = "_rt-async-std"))]
    {
        crate::rt::missing_rt((host, port, with_socket))
    }
}

/// Connect a Unix Domain Socket at the given path.
///
/// Returns an error if Unix Domain Sockets are not supported on this platform.
pub async fn connect_uds<P: AsRef<Path>, Ws: WithSocket>(
    path: P,
    with_socket: Ws,
) -> crate::Result<Ws::Output> {
    #[cfg(unix)]
    {
        #[cfg(feature = "_rt-tokio")]
        if crate::rt::rt_tokio::available() {
            use tokio::net::UnixStream;

            let stream = UnixStream::connect(path).await?;

            return Ok(with_socket.with_socket(stream));
        }

        #[cfg(feature = "_rt-async-std")]
        {
            use async_io::Async;
            use std::os::unix::net::UnixStream;

            let stream = Async::<UnixStream>::connect(path).await?;

            Ok(with_socket.with_socket(stream))
        }

        #[cfg(not(feature = "_rt-async-std"))]
        {
            crate::rt::missing_rt((path, with_socket))
        }
    }

    #[cfg(not(unix))]
    {
        drop((path, with_socket));

        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unix domain sockets are not supported on this platform",
        )
        .into())
    }
}
