use crate::error::Error;
use crate::io::Encode;
use bytes::{BufMut, Bytes, BytesMut};
use sqlx_rt::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

/// A buffered I/O wrapper around an async read/write stream.
pub struct BufStream<S> {
    inner: S,

    // read buffer
    rbuf: BytesMut,

    // write buffer
    wbuf: Vec<u8>,

    // offset into the write buffer that a previous write operation has written to
    wbuf_offset: usize,
}

impl<S> BufStream<S> {
    pub fn with_capacity(inner: S, read: usize, write: usize) -> Self {
        Self {
            inner,
            rbuf: BytesMut::with_capacity(read),
            wbuf: Vec::with_capacity(write),
            wbuf_offset: 0,
        }
    }

    pub fn write<'en, T: Encode<'en>>(&mut self, packet: T) -> Result<(), Error> {
        // we must be sure to not call [write] directly after a dropped [flush]
        debug_assert_eq!(self.wbuf_offset, 0);

        packet.encode(&mut self.wbuf)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> BufStream<S> {
    pub async fn read(&mut self, offset: usize, n: usize) -> Result<Bytes, Error> {
        self.fill_buf(offset, n).await?;

        if offset != 0 {
            // drop the bytes from 0 .. offset
            let _ = self.rbuf.split_to(offset);
        }

        // and return the slice of `n` bytes
        Ok(self.rbuf.split_to(n).freeze())
    }

    pub async fn peek(&mut self, offset: usize, n: usize) -> Result<&[u8], Error> {
        self.fill_buf(offset, n).await?;

        Ok(&self.rbuf[offset..offset + n])
    }

    async fn fill_buf(&mut self, offset: usize, n: usize) -> Result<(), Error> {
        // before waiting to receive data,
        // flush the write buffer (if needed)
        if !self.wbuf.is_empty() {
            self.flush().await?;
        }

        while self.rbuf.len() < (offset + n) {
            // ensure that there is room in the read buffer; this does nothing if there is at
            // least 128 unwritten bytes in the buffer
            self.rbuf.reserve(n.max(128));

            #[allow(unsafe_code)]
            unsafe {
                // get a chunk of uninitialized memory to write to
                // this is UB if the Read impl of the stream reads the write buffer
                let b = self.rbuf.bytes_mut();
                let b = UnsafeSend(&mut *(b as *mut [MaybeUninit<u8>] as *mut [u8]));

                // read as much as we can and return when the stream or our buffer is exhausted
                let n = self.inner.read(b.0).await?;

                // [!] read more than the length of our buffer
                debug_assert!(n <= b.0.len());

                // update the `len` of the read buffer
                self.rbuf.advance_mut(n);
            };
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        // write as much as we can each time and move the cursor as we write from the buffer
        // if _this_ future drops, offset will have a record of how much of the wbuf has
        // been written
        while self.wbuf_offset < self.wbuf.len() {
            self.wbuf_offset += self.inner.write(&self.wbuf[self.wbuf_offset..]).await?;
        }

        // fully written buffer, move cursor back to the beginning
        self.wbuf_offset = 0;
        self.wbuf.clear();

        Ok(())
    }
}

struct UnsafeSend<'a>(&'a mut [u8]);

// TODO? unsafe impl Send for UnsafeSend<'_> {}

impl<S> Deref for BufStream<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> DerefMut for BufStream<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
