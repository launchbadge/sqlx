use std::io;

use bytes::{Buf, Bytes};
use memchr::memchr;

use crate::error::Error;

pub trait ReadExt {
    fn read_bytes_with_nul(&mut self) -> Result<Bytes, Error>;

    fn read_str_with_nul(&mut self) -> Result<String, Error>;
}

impl ReadExt for Bytes {
    fn read_bytes_with_nul(&mut self) -> Result<Bytes, Error> {
        let end =
            memchr(b'\0', self).ok_or_else(|| err_protocol!("expected NUL in byte sequence"))?;

        let bytes = self.slice(..end);

        self.advance(end + 1);

        Ok(bytes)
    }

    fn read_str_with_nul(&mut self) -> Result<String, Error> {
        let bytes = self.read_bytes_with_nul()?;

        let s = std::str::from_utf8(&*bytes)
            .map_err(|err| err_protocol!("{}", err))?
            .to_owned();

        Ok(s)
    }
}
