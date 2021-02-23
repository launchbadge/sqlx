use std::io;

use bytes::{Buf, Bytes};
use bytestring::ByteString;
use sqlx_core::io::BufExt;

// UNSAFE: _unchecked string methods
// intended for use when the protocol is *known* to always produce
//  valid UTF-8 data

pub(crate) trait MySqlBufExt: BufExt {
    fn get_uint_lenenc(&mut self) -> u64;

    fn get_str_lenenc(&mut self) -> io::Result<ByteString>;

    fn get_str_eof(&mut self) -> io::Result<ByteString>;

    fn get_bytes_lenenc(&mut self) -> Bytes;
}

impl MySqlBufExt for Bytes {
    fn get_uint_lenenc(&mut self) -> u64 {
        // https://dev.mysql.com/doc/internals/en/integer.html#packet-Protocol::LengthEncodedInteger

        match self.get_u8() {
            // NOTE: 0xFB represents NULL in TextResultRow
            0xfb => unreachable!("unexpected 0xFB (NULL) in `get_uint_lenenc`"),

            0xfc => u64::from(self.get_u16_le()),
            0xfd => self.get_uint_le(3),
            0xfe => self.get_u64_le(),

            // NOTE: 0xFF may be the first byte of an ERR packet
            0xff => unreachable!("unexpected 0xFF (undefined) in `get_uint_lenenc`"),

            value => u64::from(value),
        }
    }

    fn get_str_lenenc(&mut self) -> io::Result<ByteString> {
        let len = self.get_uint_lenenc() as usize;

        self.get_str(len)
    }

    fn get_str_eof(&mut self) -> io::Result<ByteString> {
        self.get_str(self.len())
    }

    fn get_bytes_lenenc(&mut self) -> Bytes {
        #[allow(clippy::cast_possible_truncation)]
        let len = self.get_uint_lenenc() as usize;

        self.split_to(len)
    }
}
