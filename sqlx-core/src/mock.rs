use std::collections::HashMap;
use std::io;
#[cfg(feature = "async")]
use std::pin::Pin;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;

use bytes::BytesMut;
use conquer_once::Lazy;
use crossbeam::channel;
use parking_lot::RwLock;

#[cfg(feature = "blocking")]
use crate::blocking;
use crate::{io::Stream, Runtime};

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
}

#[cfg(feature = "async")]
impl crate::Async for Mock {
    fn connect_tcp_async(
        _host: &str,
        port: u16,
    ) -> futures_util::future::BoxFuture<'_, io::Result<Self::TcpStream>> {
        Box::pin(futures_util::future::ready(Self::get_stream(port)))
    }
}

#[cfg(feature = "blocking")]
impl crate::blocking::Runtime for Mock {
    fn connect_tcp(_host: &str, port: u16) -> io::Result<Self::TcpStream> {
        Self::get_stream(port)
    }
}

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
}

impl<'s> Stream<'s, Mock> for MockStream {
    #[cfg(feature = "async")]
    type ReadFuture = Pin<Box<dyn std::future::Future<Output = io::Result<usize>> + 's + Send>>;

    #[cfg(feature = "async")]
    type WriteFuture = Pin<Box<dyn std::future::Future<Output = io::Result<usize>> + 's + Send>>;

    #[cfg(feature = "async")]
    fn read_async(&'s mut self, mut buf: &'s mut [u8]) -> Self::ReadFuture {
        Box::pin(async move {
            use io::Write;

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
        let _ = self.write.send(buf.to_vec());

        // that was easy
        Box::pin(futures_util::future::ok(buf.len()))
    }
}

#[cfg(feature = "blocking")]
impl<'s> blocking::io::Stream<'s, Mock> for MockStream {
    fn read(&'s mut self, mut buf: &'s mut [u8]) -> io::Result<usize> {
        use io::Write;

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
            #[allow(clippy::map_err_ignore)]
            let message = self.read.recv().map_err(|_err| io::ErrorKind::ConnectionAborted)?;

            self.rbuf.extend_from_slice(&message);
            // loop around and now send out this message
        }
    }

    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize> {
        // send it all, right away
        let _ = self.write.send(buf.to_vec());

        // that was easy
        Ok(buf.len())
    }
}
