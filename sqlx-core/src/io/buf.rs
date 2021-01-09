use std::io;

use bytes::{Buf, Bytes};
use bytestring::ByteString;
use memchr::memchr;

#[allow(clippy::module_name_repetitions)]
pub trait BufExt: Buf {
    /// # Safety
    /// This function is unsafe because it does not check the bytes that are read are valid UTF-8.
    #[allow(unsafe_code)]
    unsafe fn get_str_unchecked(&mut self, n: usize) -> ByteString;

    /// # Safety
    /// This function is unsafe because it does not check the bytes that are read are valid UTF-8.
    #[allow(unsafe_code)]
    unsafe fn get_str_nul_unchecked(&mut self) -> io::Result<ByteString>;
}

impl BufExt for Bytes {
    #[allow(unsafe_code)]
    unsafe fn get_str_unchecked(&mut self, n: usize) -> ByteString {
        ByteString::from_bytes_unchecked(self.split_to(n))
    }

    #[allow(unsafe_code)]
    unsafe fn get_str_nul_unchecked(&mut self) -> io::Result<ByteString> {
        let nul = memchr(b'\0', self).ok_or(io::ErrorKind::InvalidData)?;

        Ok(ByteString::from_bytes_unchecked(self.split_to(nul + 1).slice(..nul)))
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use bytes::{Buf, Bytes};

    use super::BufExt;

    #[test]
    fn test_get_str() {
        let mut buf = Bytes::from_static(b"Hello World\0");

        #[allow(unsafe_code)]
        let s = unsafe { buf.get_str_unchecked(5) };

        buf.advance(1);

        #[allow(unsafe_code)]
        let s2 = unsafe { buf.get_str_unchecked(5) };

        assert_eq!(&s, "Hello");
        assert_eq!(&s2, "World");
    }

    #[test]
    fn test_get_str_nul() -> io::Result<()> {
        let mut buf = Bytes::from_static(b"Hello\0 World\0");

        #[allow(unsafe_code)]
        let s = unsafe { buf.get_str_nul_unchecked()? };

        buf.advance(1);

        #[allow(unsafe_code)]
        let s2 = unsafe { buf.get_str_nul_unchecked()? };

        assert_eq!(&s, "Hello");
        assert_eq!(&s2, "World");

        Ok(())
    }
}
