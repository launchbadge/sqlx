use async_std::io::{
    prelude::{ReadExt, WriteExt},
    Read, Write,
};
use bytes::{BufMut, BytesMut};
use std::io;

pub struct BufStream<S> {
    pub(crate) stream: S,

    // Have we reached end-of-file (been disconnected)
    stream_eof: bool,

    // Buffer used when sending outgoing messages
    wbuf: Vec<u8>,

    // Buffer used when reading incoming messages
    rbuf: BytesMut,
}

impl<S> BufStream<S>
where
    S: Read + Write + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            stream_eof: false,
            wbuf: Vec::with_capacity(1024),
            rbuf: BytesMut::with_capacity(8 * 1024),
        }
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.wbuf
    }

    #[inline]
    pub async fn flush(&mut self) -> io::Result<()> {
        if !self.wbuf.is_empty() {
            self.stream.write_all(&self.wbuf).await?;
            self.wbuf.clear();
        }

        Ok(())
    }

    #[inline]
    pub fn consume(&mut self, cnt: usize) {
        self.rbuf.advance(cnt);
    }

    pub async fn peek(&mut self, cnt: usize) -> io::Result<Option<&[u8]>> {
        loop {
            // Reaching end-of-file (read 0 bytes) will continuously
            // return None from all future calls to read
            if self.stream_eof {
                return Ok(None);
            }

            // If we have enough bytes in our read buffer,
            // return immediately
            if self.rbuf.len() >= cnt {
                return Ok(Some(&self.rbuf[..cnt]));
            }

            if self.rbuf.capacity() < cnt {
                // Ask for exactly how much we need with a lower bound of 32-bytes
                let needed = (cnt - self.rbuf.capacity()).max(32);
                self.rbuf.reserve(needed);
            }

            // SAFE: Read data in directly to buffer without zero-initializing the data.
            //       Postgres is a self-describing format and the TCP frames encode
            //       length headers. We will never attempt to decode more than we
            //       received.
            let n = self.stream.read(unsafe { self.rbuf.bytes_mut() }).await?;

            // SAFE: After we read in N bytes, we can tell the buffer that it actually
            //       has that many bytes MORE for the decode routines to look at
            unsafe { self.rbuf.advance_mut(n) }

            if n == 0 {
                self.stream_eof = true;
            }
        }
    }
}

// TODO: Find a nicer way to do this
// Return `Ok(None)` immediately from a function if the wrapped value is `None`
#[allow(unused)]
macro_rules! ret_if_none {
    ($val:expr) => {
        match $val {
            Some(val) => val,
            None => {
                return Ok(None);
            }
        }
    };
}
