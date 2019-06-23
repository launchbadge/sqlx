use bytes::Bytes;
use memchr::memchr;
use std::{io, str};

pub trait Decode {
    fn decode(src: Bytes) -> io::Result<Self>
    where
        Self: Sized;
}

#[inline]
pub(crate) fn get_str(src: &[u8]) -> io::Result<&str> {
    let end = memchr(b'\0', &src).ok_or(io::ErrorKind::UnexpectedEof)?;
    let buf = &src[..end];
    let s = str::from_utf8(buf).map_err(|_| io::ErrorKind::InvalidData)?;

    Ok(s)
}

#[inline]
pub(crate) fn get_str_bytes_unchecked(src: &Bytes) -> Bytes {
    let end = memchr(b'\0', &src).unwrap();
    let buf = src.slice_to(end);

    buf
}
