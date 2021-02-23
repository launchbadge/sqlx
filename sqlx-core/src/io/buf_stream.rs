use std::marker::PhantomData;
use std::cmp;
use std::ops::{Deref, DerefMut};

use bytes::{Bytes, BytesMut};

use super::Stream;

/// Wraps a stream and buffers input and output to and from it.
///
/// It can be excessively inefficient to work directly with a `Read` or `Write`. For example,
/// every call to `read` or `write` on `TcpStream` results in a system call (leading to
/// a network interaction). `BufStream` keeps a read and write buffer with infrequent calls
/// to `read` and `write` on the underlying stream.
///
pub struct BufStream<Rt, S> {
    runtime: PhantomData<Rt>,

    #[cfg_attr(not(any(feature = "async", feature = "blocking")), allow(unused))]
    stream: S,

    // (r)ead buffer
    rbuf: BytesMut,

    // (w)rite buffer
    wbuf: Vec<u8>,

    // offset into [wbuf] that a previous write operation has written into
    wbuf_offset: usize,
}

impl<Rt, S> BufStream<Rt, S> {
    pub fn with_capacity(stream: S, read: usize, write: usize) -> Self {
        Self {
            stream,
            runtime: PhantomData,
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
        let _rem = self.take(n);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.wbuf.reserve(additional);
    }

    pub fn write(&mut self, buf: &[u8]) {
        self.wbuf.extend_from_slice(buf);
    }

    // returns a mutable reference to the write buffer
    pub fn buffer(&mut self) -> &mut Vec<u8> {
        &mut self.wbuf
    }
}

macro_rules! read {
    ($(@$blocking:ident)? $self:ident, $offset:ident, $n:ident) => {{
        use bytes::BufMut;

        // before waiting to receive data; ensure that the write buffer is flushed
        if !$self.wbuf.is_empty() {
            read!($(@$blocking)? @flush $self);
        }

        // while our read buffer is too small to satisfy the requested amount
        while $self.rbuf.len() < ($offset + $n) {
            // ensure that there is room in the read buffer
            $self.rbuf.reserve(cmp::max($n, 128));

            #[allow(unsafe_code)]
            unsafe {
                // prepare a chunk of uninitialized memory to write to
                // this is UB if the Read impl of the stream reads from the write buffer
                let b = $self.rbuf.chunk_mut();
                let b = std::slice::from_raw_parts_mut(b.as_mut_ptr(), b.len());

                // read as much as we can and return when the stream or our buffer is exhausted
                let read = read!($(@$blocking)? @read $self, b);

                // [!] read more than the length of our buffer
                debug_assert!(read <= b.len());

                // update the len of the read buffer to let the safe world that its okay
                // to look at these bytes now
                $self.rbuf.advance_mut(read);
            }
        }

        log::trace!(
            "read  [{:>4}] > {:?}",
            $n,
            bytes::Bytes::copy_from_slice(&$self.rbuf[$offset..($offset+$n)]),
        );

        Ok(())
    }};

    (@blocking @flush $self:ident) => {
        $self.flush()?
    };

    (@flush $self:ident) => {
        $self.flush_async().await?
    };

    (@blocking @read $self:ident, $b:ident) => {
        $self.stream.read($b)?
    };

    (@read $self:ident, $b:ident) => {
        $self.stream.read_async($b).await?
    };
}

macro_rules! flush {
    ($(@$blocking:ident)? $self:ident) => {{
        log::trace!(
            "write [{:>4}] > {:?}",
            $self.wbuf.len(),
            bytes::Bytes::copy_from_slice(&$self.wbuf)
        );

        // write as much as we can each time and move the cursor as we write from the buffer
        // if _this_ future drops, offset will have a record of how much of the wbuf has
        // been written
        while $self.wbuf_offset < $self.wbuf.len() {
            $self.wbuf_offset += flush!($(@$blocking)? @write $self, &$self.wbuf[$self.wbuf_offset..]);
        }

        // fully written buffer, move cursor back to the beginning
        $self.wbuf_offset = 0;
        $self.wbuf.clear();

        Ok(())
    }};

    (@blocking @write $self:ident, $b:expr) => {
        $self.stream.write($b)?
    };

    (@write $self:ident, $b:expr) => {
        $self.stream.write_async($b).await?
    };
}

#[cfg(feature = "async")]
impl<Rt, S> BufStream<Rt, S>
where
    Rt: crate::Async,
    S: for<'s> Stream<'s, Rt>,
{
    pub async fn flush_async(&mut self) -> crate::Result<()> {
        flush!(self)
    }

    pub async fn read_async(&mut self, offset: usize, n: usize) -> crate::Result<()> {
        read!(self, offset, n)
    }
}

#[cfg(feature = "blocking")]
impl<Rt, S> BufStream<Rt, S>
where
    Rt: crate::blocking::Runtime,
    S: for<'s> Stream<'s, Rt>,
{
    pub fn flush(&mut self) -> crate::Result<()> {
        flush!(@blocking self)
    }

    pub fn read(&mut self, offset: usize, n: usize) -> crate::Result<()> {
        read!(@blocking self, offset, n)
    }
}

impl<Rt, S> Deref for BufStream<Rt, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<Rt, S> DerefMut for BufStream<Rt, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
