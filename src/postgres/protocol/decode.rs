use memchr::memchr;
use std::{convert::TryInto, io, str};

pub trait Decode {
    fn decode(src: &[u8]) -> io::Result<Self>
    where
        Self: Sized;
}

#[inline]
pub(crate) fn get_str(src: &[u8]) -> &str {
    let end = memchr(b'\0', &src).expect("expected null terminator in UTF-8 string");
    let buf = &src[..end];

    unsafe { str::from_utf8_unchecked(buf) }
}

pub trait Buf {
    fn advance(&mut self, cnt: usize);

    // An n-bit integer in network byte order (IntN)
    fn get_u8(&mut self) -> io::Result<u8>;
    fn get_u16(&mut self) -> io::Result<u16>;
    fn get_i32(&mut self) -> io::Result<i32>;
    fn get_u32(&mut self) -> io::Result<u32>;

    // A null-terminated string
    fn get_str_null(&mut self) -> io::Result<&str>;
}

impl<'a> Buf for &'a [u8] {
    fn advance(&mut self, cnt: usize) {
        *self = &self[cnt..];
    }

    fn get_u8(&mut self) -> io::Result<u8> {
        let val = self[0];

        self.advance(1);

        Ok(val)
    }

    fn get_u16(&mut self) -> io::Result<u16> {
        let val: [u8; 2] = (&self[..2])
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        self.advance(2);

        Ok(u16::from_be_bytes(val))
    }

    fn get_i32(&mut self) -> io::Result<i32> {
        let val: [u8; 4] = (&self[..4])
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        self.advance(4);

        Ok(i32::from_be_bytes(val))
    }

    fn get_u32(&mut self) -> io::Result<u32> {
        let val: [u8; 4] = (&self[..4])
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        self.advance(4);

        Ok(u32::from_be_bytes(val))
    }

    fn get_str_null(&mut self) -> io::Result<&str> {
        let end = memchr(b'\0', &*self).ok_or(io::ErrorKind::InvalidData)?;
        let buf = &self[..end];

        self.advance(end + 1);

        if cfg!(debug_asserts) {
            str::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        } else {
            Ok(unsafe { str::from_utf8_unchecked(buf) })
        }
    }
}
