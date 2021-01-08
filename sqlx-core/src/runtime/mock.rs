use std::collections::HashMap;
#[cfg(feature = "async")]
use std::pin::Pin;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
#[cfg(feature = "async")]
use std::task::{Context, Poll};

use bytes::BytesMut;
use conquer_once::Lazy;
use crossbeam::channel;
use parking_lot::RwLock;

use crate::Runtime;

#[derive(Debug)]
#[doc(hidden)]
pub struct Mock;

#[derive(Debug)]
#[doc(hidden)]
pub struct MockStream {
    port: u16,
    rbuf: BytesMut,
    read: channel::Receiver<Vec<u8>>,
    write: channel::Sender<Vec<u8>>,
}

static MOCK_STREAM_PORT: AtomicU16 = AtomicU16::new(0);

static MOCK_STREAMS: Lazy<RwLock<HashMap<u16, MockStream>>> = Lazy::new(RwLock::default);

impl Runtime for Mock {
    type TcpStream = MockStream;
}

#[cfg(feature = "async")]
impl crate::AsyncRuntime for Mock {
    fn connect_tcp(
        _host: &str,
        port: u16,
    ) -> futures_util::future::BoxFuture<'_, std::io::Result<Self::TcpStream>> {
        Box::pin(match MOCK_STREAMS.write().remove(&port) {
            Some(stream) => futures_util::future::ok(stream),
            None => futures_util::future::err(std::io::ErrorKind::ConnectionRefused.into()),
        })
    }
}

#[cfg(feature = "blocking")]
impl crate::blocking::Runtime for Mock {
    fn connect_tcp(_host: &str, port: u16) -> std::io::Result<Self::TcpStream> {
        match MOCK_STREAMS.write().remove(&port) {
            Some(stream) => Ok(stream),
            None => Err(std::io::ErrorKind::ConnectionRefused.into()),
        }
    }
}

impl Mock {
    pub fn stream() -> MockStream {
        let port = MOCK_STREAM_PORT.fetch_add(1, Ordering::SeqCst) + 1;

        let (write_l, write_r) = channel::unbounded();
        let (read_r, read_l) = channel::unbounded();

        let stream_l = MockStream { port, read: read_l, write: write_l, rbuf: BytesMut::new() };
        let stream_r = MockStream { port, read: write_r, write: read_r, rbuf: BytesMut::new() };

        MOCK_STREAMS.write().insert(port, stream_l);

        stream_r
    }
}

impl MockStream {
    pub fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(feature = "blocking")]
impl std::io::Read for MockStream {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Write;

        loop {
            if !self.rbuf.is_empty() {
                // write as much data from our read buffer as we can
                let written = buf.write(&self.rbuf)?;

                // remove the bytes that we were able to write
                let _ = self.rbuf.split_to(written);

                // return how many bytes we wrote
                return Ok(written);
            }

            // no bytes in the buffer, ask the channel for more
            let message = self.read.recv().map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

            self.rbuf.extend_from_slice(&message);
            // loop around and now send out this message
        }
    }
}

#[cfg(feature = "blocking")]
impl std::io::Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // send it all, right away
        let _ = self.write.send(buf.to_vec());

        // that was easy
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // no implementation needed
        // flush is inherent
        Ok(())
    }
}

#[cfg(feature = "async")]
impl futures_io::AsyncRead for MockStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        use std::io::Write;

        loop {
            if !self.rbuf.is_empty() {
                // write as much data from our read buffer as we can
                let written = buf.write(&self.rbuf)?;

                // remove the bytes that we were able to write
                let _ = self.rbuf.split_to(written);

                // return how many bytes we wrote
                return Poll::Ready(Ok(written));
            }

            // no bytes in the buffer, ask the channel for more
            let message = if let Ok(message) = self.read.try_recv() {
                message
            } else {
                // no data, return pending (and immediately wake again to run try_recv again)
                cx.waker().wake_by_ref();
                return Poll::Pending;
            };

            self.rbuf.extend_from_slice(&message);
            // loop around and now send out this message
        }
    }
}

#[cfg(feature = "async")]
impl futures_io::AsyncWrite for MockStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        // send it all, right away
        let _ = self.write.send(buf.to_vec());

        // that was easy
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // no implementation needed
        // flush is inherent
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // nothing happens, ha
        Poll::Ready(Ok(()))
    }
}
