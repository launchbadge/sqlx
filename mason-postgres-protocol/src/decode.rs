use bytes::Bytes;
use memchr::memchr;
use std::{io, str};

pub trait Decode {
    fn decode(b: Bytes) -> io::Result<Self>
    where
        Self: Sized;
}

#[inline]
pub(crate) fn get_str<'a>(b: &'a [u8]) -> io::Result<&'a str> {
    let end = memchr(b'\0', &b).ok_or(io::ErrorKind::UnexpectedEof)?;
    let buf = &b[..end];
    let s = str::from_utf8(buf).map_err(|_| io::ErrorKind::InvalidData)?;

    Ok(s)
}
