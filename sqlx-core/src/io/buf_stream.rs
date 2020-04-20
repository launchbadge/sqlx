use std::io;
use std::ops::{Deref, DerefMut};

use bytes::BytesMut;

use crate::error::Error;
use crate::io::{decode::Decode, encode::Encode};
use crate::runtime::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream: S,

    // writes with `write` to the underlying stream are buffered
    // this can be flushed with `flush`
    wbuf: Vec<u8>,

    // we read into the read buffer using 100% safe code
    // this requires us to store the "real" length as `rbuf.len()`
    // is now the capacity
    rbuf: BytesMut,
    rbuf_len: usize,
}

impl<S> BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            wbuf: Vec::with_capacity(512),
            rbuf: BytesMut::with_capacity(1024),
            rbuf_len: 0,
        }
    }

    pub async fn write<T>(&mut self, value: T) -> Result<(), Error>
    where
        T: Encode,
    {
        // all writes are done on an empty buffer; this greatly simplifies encoding logic
        // clearing a Vec is O(1) for Copy typees, it just sets the len to 0
        self.wbuf.clear();

        value.encode(&mut self.wbuf);

        // this empty buffer is then written to the stream that is internally buffered
        self.stream.write_all(&self.wbuf).await?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        self.stream.flush().await.map_err(Into::into)
    }

    pub async fn read<T>(&mut self, cnt: usize) -> Result<T, Error>
    where
        T: Decode,
    {
        if (self.rbuf.len() - self.rbuf_len) < cnt {
            // not enough space remaining in our read buffer
            // re-allocate the buffer to a larger size
            // this uses the 2x growing rule so it will allocate much more space than
            // we _strictly_ need, which is a good thing
            self.rbuf.resize(self.rbuf.len() + cnt, 0);
        }

        while cnt > self.rbuf_len {
            let n = self.stream.read(&mut *self.rbuf).await?;

            if n == 0 {
                // a zero read when we had space in the read buffer
                // should be treated as an EOF

                // and an unexpected EOF means the server told us to go away

                return Err(io::Error::from(io::ErrorKind::ConnectionAborted).into());
            }

            self.rbuf_len += n;
        }

        self.rbuf_len -= cnt;

        T::decode(self.rbuf.split_to(cnt).freeze())
    }
}

impl<S> Deref for BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<S> DerefMut for BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
