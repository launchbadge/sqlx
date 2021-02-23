use std::convert::TryFrom;
use std::io;

use bytes::{Buf, Bytes};
use bytestring::ByteString;
use memchr::memchr;

#[allow(clippy::module_name_repetitions)]
pub trait BufExt: Buf {
    fn get_str(&mut self, n: usize) -> io::Result<ByteString>;

    fn get_str_nul(&mut self) -> io::Result<ByteString>;
}

impl BufExt for Bytes {
    fn get_str(&mut self, n: usize) -> io::Result<ByteString> {
        ByteString::try_from(self.split_to(n))
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    fn get_str_nul(&mut self) -> io::Result<ByteString> {
        let nul = memchr(b'\0', self).ok_or(io::ErrorKind::InvalidData)?;

        ByteString::try_from(self.split_to(nul + 1).slice(..nul))
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use bytes::{Buf, Bytes};

    use super::BufExt;

    #[test]
    fn test_get_str() -> io::Result<()> {
        let mut buf = Bytes::from_static(b"Hello World\0");

        let s = buf.get_str(5)?;

        buf.advance(1);

        let s2 = buf.get_str(5)?;

        assert_eq!(&s, "Hello");
        assert_eq!(&s2, "World");

        Ok(())
    }

    #[test]
    fn test_get_str_nul() -> io::Result<()> {
        let mut buf = Bytes::from_static(b"Hello\0 World\0");

        let s = buf.get_str_nul()?;

        buf.advance(1);

        let s2 = buf.get_str_nul()?;

        assert_eq!(&s, "Hello");
        assert_eq!(&s2, "World");

        Ok(())
    }
}
