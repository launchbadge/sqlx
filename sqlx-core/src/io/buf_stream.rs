use std::io;
use std::ops::{Deref, DerefMut};

use crate::runtime::{AsyncRead, AsyncReadExt, AsyncWriteExt, AsyncWrite};

const RBUF_SIZE: usize = 8 * 1024;

pub struct BufStream<S> {
    pub(crate) stream: S,

    // Have we reached end-of-file (been disconnected)
    stream_eof: bool,

    // Buffer used when sending outgoing messages
    wbuf: Vec<u8>,

    // Buffer used when reading incoming messages
    rbuf: Vec<u8>,
    rbuf_rindex: usize,
    rbuf_windex: usize,
}

impl<S> BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            stream_eof: false,
            wbuf: Vec::with_capacity(1024),
            rbuf: vec![0; RBUF_SIZE],
            rbuf_rindex: 0,
            rbuf_windex: 0,
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

    pub fn clear_bufs(&mut self) {
        self.rbuf_rindex = 0;
        self.rbuf_windex = 0;
        self.wbuf.clear();
    }

    #[inline]
    pub fn consume(&mut self, cnt: usize) {
        self.rbuf_rindex += cnt;
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
            if self.rbuf_windex >= (self.rbuf_rindex + cnt) {
                let buf = &self.rbuf[self.rbuf_rindex..(self.rbuf_rindex + cnt)];

                return Ok(Some(buf));
            }

            // If we are out of space to write to in the read buffer ..
            if self.rbuf.len() < (self.rbuf_windex + cnt) {
                if self.rbuf_rindex == self.rbuf_windex {
                    // We have consumed all data; simply reset the indexes
                    self.rbuf_rindex = 0;
                    self.rbuf_windex = 0;
                } else {
                    // Allocate a new buffer
                    let mut new_rbuf = Vec::with_capacity(RBUF_SIZE);

                    // Take the minimum of the read and write indexes
                    let min_index = self.rbuf_rindex.min(self.rbuf_windex);

                    // Copy the old buffer to our new buffer
                    new_rbuf.extend_from_slice(&self.rbuf[min_index..]);

                    // Zero-extend the new buffer
                    new_rbuf.resize(new_rbuf.capacity(), 0);

                    // Replace the old buffer with our new buffer
                    self.rbuf = new_rbuf;

                    // And reduce the indexes
                    self.rbuf_rindex -= min_index;
                    self.rbuf_windex -= min_index;
                }

                // Do we need more space still
                if self.rbuf.len() < (self.rbuf_windex + cnt) {
                    let needed = (self.rbuf_windex + cnt) - self.rbuf.len();

                    self.rbuf.resize(self.rbuf.len() + needed, 0);
                }
            }

            let n = self.stream.read(&mut self.rbuf[self.rbuf_windex..]).await?;

            self.rbuf_windex += n;

            if n == 0 {
                self.stream_eof = true;
            }
        }
    }
}

impl<S> Deref for BufStream<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<S> DerefMut for BufStream<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
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
