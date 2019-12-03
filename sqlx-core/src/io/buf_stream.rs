use async_std::io::{
    prelude::{ReadExt, WriteExt},
    Read, Write,
};
use std::io;
use bitflags::_core::mem::MaybeUninit;

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
        S: Read + Write + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            stream_eof: false,
            wbuf: Vec::with_capacity(1024),
            rbuf: vec![0; 8 * 1024],
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
                return Ok(Some(&self.rbuf[self.rbuf_rindex..(self.rbuf_rindex + cnt)]));
            }

            // If we are out of space to write to in the read buffer,
            // we reset the indexes
            if self.rbuf.len() < (self.rbuf_windex + cnt) {
                // TODO: This assumes that all data is consumed when we need to re-allocate
                debug_assert_eq!(self.rbuf_rindex, self.rbuf_windex);

                self.rbuf_rindex = 0;
                self.rbuf_windex = 0;
            }

            let n = self.stream.read(&mut self.rbuf[self.rbuf_windex..]).await?;

            self.rbuf_windex += n;

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
