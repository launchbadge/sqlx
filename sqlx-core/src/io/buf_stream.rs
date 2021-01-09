use std::io;
use std::ops::{Deref, DerefMut, RangeBounds};

use bytes::BytesMut;
use futures::lock::Mutex;
use sqlx_rt::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::error::Error;
use crate::io::write_and_flush::WriteAndFlush;
use crate::io::{decode::Decode, encode::Encode};
use std::io::Cursor;

pub struct BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream: S,

    // writes with `write` to the underlying stream are buffered
    // this can be flushed with `flush`
    pub(crate) wbuf: Vec<u8>,

    // we read into the read buffer using 100% safe code
    rbuf: BytesMut,
    rlock: Mutex<()>,
}

impl<S> BufStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            wbuf: Vec::with_capacity(512),
            rbuf: BytesMut::with_capacity(4096),
            rlock: Mutex::new(()),
        }
    }

    pub fn write<'en, T>(&mut self, value: T)
    where
        T: Encode<'en, ()>,
    {
        self.write_with(value, ())
    }

    pub fn write_with<'en, T, C>(&mut self, value: T, context: C)
    where
        T: Encode<'en, C>,
    {
        value.encode_with(&mut self.wbuf, context);
    }

    pub fn flush(&mut self) -> WriteAndFlush<'_, S> {
        WriteAndFlush {
            stream: &mut self.stream,
            buf: Cursor::new(&mut self.wbuf),
        }
    }

    pub async fn read<'de, T>(&mut self, cnt: usize) -> Result<T, Error>
    where
        T: Decode<'de, ()>,
    {
        self.read_with(cnt, ()).await
    }

    pub async fn read_with<'de, T, C>(&mut self, cnt: usize, context: C) -> Result<T, Error>
    where
        T: Decode<'de, C>,
    {
        T::decode_with(self.read_raw(cnt).await?.freeze(), context)
    }

    pub async fn read_raw(&mut self, cnt: usize) -> Result<BytesMut, Error> {
        let _lock = self.rlock.lock().await;
        read_raw_into(&mut self.stream, &mut self.rbuf, cnt).await?;
        Ok(self.rbuf.split_to(cnt))
    }

    // pub async fn read_raw_into(&mut self, buf: &mut BytesMut, cnt: usize) -> Result<(), Error> {
    //     let _lock = self.rlock.lock().await;
    //     read_raw_into(&mut self.stream, buf, cnt - self.rbuf.len()).await
    // }

    pub async fn slice(&mut self, range: std::ops::Range<usize>) -> Result<&[u8], Error> {
        use core::ops::Bound;
        let _lock = self.rlock.lock().await;
        let len = self.rbuf.len();
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };
        if end > len {
            read_raw_into(&mut self.stream, &mut self.rbuf, end - len).await?;
        }
        let slice: &[u8] = self.rbuf.as_ref();
        Ok(&slice[range])
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

async fn read_raw_into<S: AsyncRead + Unpin>(
    stream: &mut S,
    buf: &mut BytesMut,
    cnt: usize,
) -> Result<(), Error> {
    let offset = buf.len();

    // zero-fills the space in the read buffer
    buf.resize(cnt, 0);

    let mut read = offset;
    while cnt > read {
        // read in bytes from the stream into the read buffer starting
        // from the offset we last read from
        let n = stream.read(&mut buf[read..]).await?;

        if n == 0 {
            // a zero read when we had space in the read buffer
            // should be treated as an EOF

            // and an unexpected EOF means the server told us to go away

            return Err(io::Error::from(io::ErrorKind::ConnectionAborted).into());
        }

        read += n;
    }
    Ok(())
}
