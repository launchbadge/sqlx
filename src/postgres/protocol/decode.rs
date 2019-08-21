use memchr::memchr;
use std::str;

pub trait Decode {
    fn decode(src: &[u8]) -> Self
    where
        Self: Sized;
}

#[inline]
pub(crate) fn get_str(src: &[u8]) -> &str {
    let end = memchr(b'\0', &src).expect("expected null terminator in UTF-8 string");
    let buf = &src[..end];

    unsafe { str::from_utf8_unchecked(buf) }
}
