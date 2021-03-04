use std::collections::HashMap;
use std::io;
#[cfg(unix)]
use std::path::Path;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;

use bytes::BytesMut;
use conquer_once::Lazy;
use crossbeam::channel;
#[cfg(feature = "async")]
use futures_util::future::{self, BoxFuture};
use parking_lot::RwLock;

use crate::{io::Stream as IoStream, Runtime};

#[derive(Debug)]
pub struct Mock;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
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

    #[cfg(unix)]
    type UnixStream = MockStream;

    #[doc(hidden)]
    #[cfg(feature = "blocking")]
    fn connect_tcp(_host: &str, port: u16) -> io::Result<Self::TcpStream> {
        Self::get_stream(port)
    }

    #[doc(hidden)]
    #[cfg(all(unix, feature = "blocking"))]
    fn connect_unix(_path: &Path) -> io::Result<Self::UnixStream> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Unix streams are not supported in the Mock runtime",
        ))
    }

    #[doc(hidden)]
    #[cfg(feature = "async")]
    fn connect_tcp_async(_host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        Box::pin(future::ready(Self::get_stream(port)))
    }

    #[doc(hidden)]
    #[cfg(all(unix, feature = "async"))]
    fn connect_unix_async(_path: &Path) -> BoxFuture<'_, io::Result<Self::UnixStream>> {
        Box::pin(future::err(io::Error::new(
            io::ErrorKind::Other,
            "Unix streams are not supported in the Mock runtime",
        )))
    }
}

#[cfg(feature = "blocking")]
impl crate::blocking::Runtime for Mock {}

impl Mock {
    #[must_use]
    pub fn stream() -> MockStream {
        let port = MOCK_STREAM_PORT.fetch_add(1, Ordering::SeqCst) + 1;

        let (write_l, write_r) = channel::unbounded();
        let (read_r, read_l) = channel::unbounded();

        let stream_l = MockStream { port, read: read_l, write: write_l, rbuf: BytesMut::new() };
        let stream_r = MockStream { port, read: write_r, write: read_r, rbuf: BytesMut::new() };

        MOCK_STREAMS.write().insert(port, stream_l);

        stream_r
    }

    fn get_stream(port: u16) -> io::Result<MockStream> {
        match MOCK_STREAMS.write().remove(&port) {
            Some(stream) => Ok(stream),
            None => Err(io::ErrorKind::ConnectionRefused.into()),
        }
    }
}

impl MockStream {
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rbuf.is_empty() && self.read.is_empty() && self.write.is_empty()
    }
}

#[cfg(feature = "async")]
impl crate::Async for Mock {}

impl<'s> IoStream<'s, Mock> for MockStream {
    #[cfg(feature = "async")]
    type ReadFuture = BoxFuture<'s, io::Result<usize>>;

    #[cfg(feature = "async")]
    type WriteFuture = BoxFuture<'s, io::Result<usize>>;

    #[cfg(feature = "async")]
    type ShutdownFuture = future::Ready<io::Result<()>>;

    #[cfg(feature = "async")]
    fn read_async(&'s mut self, mut buf: &'s mut [u8]) -> Self::ReadFuture {
        Box::pin(async move {
            use io::Write;

            loop {
                if !self.rbuf.is_empty() {
                    // write as much data from our read buffer as we can
                    let written = buf.write(&self.rbuf)?;

                    // remove the bytes that we were able to write
                    let _rem = self.rbuf.split_to(written);

                    // return how many bytes we wrote
                    return Ok(written);
                }

                // no bytes in the buffer, ask the channel for more
                let message = if let Ok(message) = self.read.try_recv() {
                    message
                } else {
                    // no data, return pending (and immediately wake again to run try_recv again)
                    futures_util::pending!();
                    continue;
                };

                self.rbuf.extend_from_slice(&message);
                // loop around and now send out this message
            }
        })
    }

    #[cfg(feature = "async")]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        // send it all, right away
        let _res = self.write.send(buf.to_vec());

        // that was easy
        Box::pin(future::ok(buf.len()))
    }

    #[cfg(feature = "async")]
    fn shutdown_async(&'s mut self) -> Self::ShutdownFuture {
        future::ok(())
    }

    #[cfg(feature = "blocking")]
    fn read(&'s mut self, mut buf: &'s mut [u8]) -> io::Result<usize> {
        use io::Write;

        loop {
            if !self.rbuf.is_empty() {
                // write as much data from our read buffer as we can
                let written = buf.write(&self.rbuf)?;

                // remove the bytes that we were able to write
                let _rem = self.rbuf.split_to(written);

                // return how many bytes we wrote
                return Ok(written);
            }

            // no bytes in the buffer, ask the channel for more
            #[allow(clippy::map_err_ignore)]
            let message = self.read.recv().map_err(|_err| io::ErrorKind::ConnectionAborted)?;

            self.rbuf.extend_from_slice(&message);
            // loop around and now send out this message
        }
    }

    #[cfg(feature = "blocking")]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        // send it all, right away
        let _res = self.write.send(buf.to_vec());

        // that was easy
        Ok(buf.len())
    }

    #[cfg(feature = "blocking")]
    fn shutdown(&'s mut self) -> io::Result<()> {
        Ok(())
    }
}
