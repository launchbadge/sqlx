use bytes::Bytes;
use memchr::memchr;
use std::{io, str};

pub trait Decode {
    fn decode(src: &[u8]) -> io::Result<Self>
    where
        Self: Sized;
}

#[inline]
pub(crate) fn get_str(src: &[u8]) -> io::Result<&str> {
    let end = memchr(b'\0', &src).ok_or(io::ErrorKind::UnexpectedEof)?;
    let buf = &src[..end];
    let s = unsafe { str::from_utf8_unchecked(buf) };

    Ok(s)
}
