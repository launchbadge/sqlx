#[cfg(feature = "blocking")]
use std::io::{Read, Write};
use std::slice::from_raw_parts_mut;

use bytes::{BufMut, Bytes, BytesMut};
#[cfg(feature = "async")]
use futures_io::{AsyncRead, AsyncWrite};
#[cfg(feature = "async")]
use futures_util::{AsyncReadExt, AsyncWriteExt};

/// Wraps a stream and buffers input and output to and from it.
///
/// It can be excessively inefficient to work directly with a `Read` or `Write`. For example,
/// every call to `read` or `write` on `TcpStream` results in a system call (leading to
/// a network interaction). `BufStream` keeps a read and write buffer with infrequent calls
/// to `read` and `write` on the underlying stream.
///
pub struct BufStream<S> {
    stream: S,

    // (r)ead buffer
    rbuf: BytesMut,

    // (w)rite buffer
    wbuf: Vec<u8>,

    // offset into [wbuf] that a previous write operation has written into
    wbuf_offset: usize,
}

impl<S> BufStream<S> {
    pub fn with_capacity(stream: S, read: usize, write: usize) -> Self {
        Self {
            stream,
            rbuf: BytesMut::with_capacity(read),
            wbuf: Vec::with_capacity(write),
            wbuf_offset: 0,
        }
    }

    pub fn get(&self, offset: usize, n: usize) -> &[u8] {
        &(self.rbuf.as_ref())[offset..(offset + n)]
    }

    pub fn take(&mut self, n: usize) -> Bytes {
        self.rbuf.split_to(n).freeze()
    }

    pub fn consume(&mut self, n: usize) {
        let _ = self.take(n);
    }
}

#[cfg(feature = "async")]
impl<S> BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn read_async(&mut self, n: usize) -> crate::Result<()> {
        // // before waiting to receive data
        // // ensure that the write buffer is flushed
        // if !self.wbuf.is_empty() {
        //     self.flush().await?;
        // }

        // while our read buffer is too small to satisfy the requested amount
        while self.rbuf.len() < n {
            // ensure that there is room in the read buffer
            self.rbuf.reserve(n.max(128));

            #[allow(unsafe_code)]
            unsafe {
                // prepare a chunk of uninitialized memory to write to
                // this is UB if the Read impl of the stream reads from the write buffer
                let b = self.rbuf.chunk_mut();
                let b = from_raw_parts_mut(b.as_mut_ptr(), b.len());

                // read as much as we can and return when the stream or our buffer is exhausted
                let n = self.stream.read(b).await?;

                // [!] read more than the length of our buffer
                debug_assert!(n <= b.len());

                // update the len of the read buffer to let the safe world that its okay
                // to look at these bytes now
                self.rbuf.advance_mut(n);
            }
        }

        Ok(())
    }
}
