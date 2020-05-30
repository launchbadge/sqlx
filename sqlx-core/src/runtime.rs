#![allow(unused_imports)]

#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

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

#[cfg(feature = "runtime-async-std")]
pub(crate) use smol_shim::{
    fs, sleep, spawn, timeout, AsyncRead, AsyncReadExt, AsyncWrite, TcpStream,
};

#[cfg(all(feature = "runtime-async-std", feature = "postgres", unix))]
pub(crate) use smol_shim::UnixStream;

#[cfg(feature = "runtime-async-std")]
mod smol_shim {
    use futures_core::Future;
    use smol::Async;
    use std::error::Error;
    use std::fmt;
    use std::io;
    use std::net::{TcpStream as StdTcpStream, ToSocketAddrs};
    use std::os::unix::net::UnixStream as StdUnixStream;
    use std::path::Path;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;

    // TODO only used with tls, check later
    pub(crate) struct TcpStream(smol::Async<StdTcpStream>);
    pub(crate) struct UnixStream(smol::Async<StdUnixStream>);

    impl TcpStream {
        pub async fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
            let mut last_err = None;
            // even async-std implementation may just block during resolution anyways?
            let addrs = addr.to_socket_addrs()?;

            for addr in addrs {
                match Async::<StdTcpStream>::connect(&addr).await {
                    Ok(stream) => {
                        return Ok(TcpStream(stream));
                    }
                    Err(err) => {
                        last_err = Some(err);
                        continue;
                    }
                }
            }

            Err(last_err.unwrap_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "could not resolve to any addresses",
                )
            }))
        }

        pub fn shutdown(&self, how: std::net::Shutdown) -> std::io::Result<()> {
            self.0.get_ref().shutdown(how)
        }
    }

    impl AsyncRead for TcpStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_read(cx, buf)
        }

        fn poll_read_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            bufs: &mut [std::io::IoSliceMut],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_read_vectored(cx, bufs)
        }
    }

    impl AsyncWrite for TcpStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
            Pin::new(&mut self.0).poll_close(cx)
        }

        fn poll_write_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            bufs: &[std::io::IoSlice],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
        }
    }

    impl UnixStream {
        pub async fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
            let stream = Async::<StdUnixStream>::connect(path).await?;
            Ok(UnixStream(stream))
        }

        pub fn shutdown(&self, how: std::net::Shutdown) -> std::io::Result<()> {
            self.0.get_ref().shutdown(how)
        }
    }

    impl AsyncRead for UnixStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_read(cx, buf)
        }

        fn poll_read_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            bufs: &mut [std::io::IoSliceMut],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_read_vectored(cx, bufs)
        }
    }

    impl AsyncWrite for UnixStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
            Pin::new(&mut self.0).poll_close(cx)
        }

        fn poll_write_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            bufs: &[std::io::IoSlice],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
        }
    }

    // TODO only used with tls, check later
    pub mod fs {
        use std::fs;
        use std::path::Path;

        pub async fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
            let path = path.as_ref().to_owned();
            smol::Task::blocking(async move { fs::read(&path) }).await
        }
    }

    pub(crate) use futures_util::io::{AsyncRead, AsyncReadExt, AsyncWrite};

    pub(crate) fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        smol::Task::spawn(future).detach()
    }

    pub(crate) async fn sleep(dur: Duration) {
        smol::Timer::after(dur).await;
    }

    pub(crate) async fn timeout<T>(
        dur: Duration,
        f: impl Future<Output = T>,
    ) -> Result<T, TimeoutError> {
        TimeoutFuture::new(f, dur).await
    }

    pin_project_lite::pin_project! {
        /// A future that times out after a duration of time.
        pub struct TimeoutFuture<F> {
            #[pin]
            future: F,
            #[pin]
            delay: smol::Timer,
        }
    }

    impl<F> TimeoutFuture<F> {
        #[allow(dead_code)]
        pub(super) fn new(future: F, dur: Duration) -> TimeoutFuture<F> {
            TimeoutFuture {
                future,
                delay: smol::Timer::after(dur),
            }
        }
    }

    impl<F: Future> Future for TimeoutFuture<F> {
        type Output = Result<F::Output, TimeoutError>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            match this.future.poll(cx) {
                Poll::Ready(v) => Poll::Ready(Ok(v)),
                Poll::Pending => match this.delay.poll(cx) {
                    Poll::Ready(_) => Poll::Ready(Err(TimeoutError { _private: () })),
                    Poll::Pending => Poll::Pending,
                },
            }
        }
    }

    /// An error returned when a future times out.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct TimeoutError {
        _private: (),
    }

    impl Error for TimeoutError {}

    impl fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            "future has timed out".fmt(f)
        }
    }
}
