#![allow(dead_code)]

use std::io;
use std::ops::{Deref, DerefMut};

use bytes::BytesMut;
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
        // zero-fills the space in the read buffer
        self.rbuf.resize(cnt, 0);

        let mut read = 0;
        while cnt > read {
            // read in bytes from the stream into the read buffer starting
            // from the offset we last read from
            let n = self.stream.read(&mut self.rbuf[read..]).await?;

            if n == 0 {
                // a zero read when we had space in the read buffer
                // should be treated as an EOF

                // and an unexpected EOF means the server told us to go away

                return Err(io::Error::from(io::ErrorKind::ConnectionAborted).into());
            }

            read += n;
        }

        T::decode_with(self.rbuf.split_to(cnt).freeze(), context)
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
