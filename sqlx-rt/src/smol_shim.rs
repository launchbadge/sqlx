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

pub use futures_util::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct TcpStream(smol::Async<StdTcpStream>);

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

#[derive(Debug)]
pub struct UnixStream(smol::Async<StdUnixStream>);

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

pub mod fs {
    #[cfg(feature = "tls")]
    use std::fs;
    #[cfg(feature = "tls")]
    use std::path::Path;

    #[cfg(feature = "tls")]
    pub async fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        let path = path.as_ref().to_owned();
        smol::Task::blocking(async move { fs::read(&path) }).await
    }
}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    smol::Task::spawn(future).detach()
}

pub async fn sleep(dur: Duration) {
    smol::Timer::after(dur).await;
}

pub async fn timeout<T>(dur: Duration, f: impl Future<Output = T>) -> Result<T, TimeoutError> {
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

// from async-std. The underlying executor (smol) is the same, so this should work the same way,
// see notes on `poll`.
pub async fn yield_now() {
    YieldNow(false).await
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    // The futures executor is implemented as a FIFO queue, so all this future
    // does is re-schedule the future back to the end of the queue, giving room
    // for other futures to progress.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
