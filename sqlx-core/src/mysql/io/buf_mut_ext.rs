use crate::io::BufMut;
use byteorder::ByteOrder;
use std::{u16, u32, u64, u8};

pub trait BufMutExt {
    fn put_uint_lenenc<T: ByteOrder, U: Into<Option<u64>>>(&mut self, val: U);

    fn put_str_lenenc<T: ByteOrder>(&mut self, val: &str);

    fn put_str(&mut self, val: &str);

    fn put_bytes(&mut self, val: &[u8]);

    fn put_bytes_lenenc<T: ByteOrder>(&mut self, val: &[u8]);
}

impl BufMutExt for Vec<u8> {
    fn put_uint_lenenc<T: ByteOrder, U: Into<Option<u64>>>(&mut self, value: U) {
        if let Some(value) = value.into() {
            // https://mariadb.com/kb/en/library/protocol-data-types/#length-encoded-integers
            if value > 0xFF_FF_FF {
                // Integer value is encoded in the next 8 bytes (9 bytes total)
                self.push(0xFE);
                self.put_u64::<T>(value);
            } else if value > u64::from(u16::MAX) {
                // Integer value is encoded in the next 3 bytes (4 bytes total)
                self.push(0xFD);
                self.put_u24::<T>(value as u32);
            } else if value > u64::from(u8::MAX) {
                // Integer value is encoded in the next 2 bytes (3 bytes total)
                self.push(0xFC);
                self.put_u16::<T>(value as u16);
            } else {
                match value {
                    // If the value is of size u8 and one of the key bytes used in length encoding
                    // we must put that single byte as a u16
                    0xFB | 0xFC | 0xFD | 0xFE | 0xFF => {
                        self.push(0xFC);
                        self.put_u16::<T>(value as u16);
                    }

                    _ => {
                        self.push(value as u8);
                    }
                }
            }
        } else {
            self.push(0xFB);
        }
    }

    #[inline]
    fn put_str(&mut self, val: &str) {
        self.put_bytes(val.as_bytes());
    }

    #[inline]
    fn put_str_lenenc<T: ByteOrder>(&mut self, val: &str) {
        self.put_uint_lenenc::<T, _>(val.len() as u64);
        self.extend_from_slice(val.as_bytes());
    }

    #[inline]
    fn put_bytes(&mut self, val: &[u8]) {
        self.extend_from_slice(val);
    }

    #[inline]
    fn put_bytes_lenenc<T: ByteOrder>(&mut self, val: &[u8]) {
        self.put_uint_lenenc::<T, _>(val.len() as u64);
        self.extend_from_slice(val);
    }
}

#[cfg(test)]
mod tests {
    use super::BufMutExt;
    use crate::io::BufMut;
    use byteorder::LittleEndian;

    // [X] it_encodes_int_lenenc_u64
    // [X] it_encodes_int_lenenc_u32
    // [X] it_encodes_int_lenenc_u24
    // [X] it_encodes_int_lenenc_u16
    // [X] it_encodes_int_lenenc_u8
    // [X] it_encodes_int_u64
    // [X] it_encodes_int_u32
    // [X] it_encodes_int_u24
    // [X] it_encodes_int_u16
    // [X] it_encodes_int_u8
    // [X] it_encodes_string_lenenc
    // [X] it_encodes_string_fix
    // [X] it_encodes_string_null
    // [X] it_encodes_string_eof
    // [X] it_encodes_byte_lenenc
    // [X] it_encodes_byte_fix
    // [X] it_encodes_byte_eof

    #[test]
    fn it_encodes_int_lenenc_none() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(None);

        assert_eq!(&buf[..], b"\xFB");
    }

    #[test]
    fn it_encodes_int_lenenc_u8() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFA as u64);

        assert_eq!(&buf[..], b"\xFA");
    }

    #[test]
    fn it_encodes_int_lenenc_u16() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(std::u16::MAX as u64);

        assert_eq!(&buf[..], b"\xFC\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u24() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFF_FF_FF as u64);

        assert_eq!(&buf[..], b"\xFD\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u64() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(std::u64::MAX);

        assert_eq!(&buf[..], b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_fb() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFB as u64);

        assert_eq!(&buf[..], b"\xFC\xFB\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFC as u64);

        assert_eq!(&buf[..], b"\xFC\xFC\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fd() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFD as u64);

        assert_eq!(&buf[..], b"\xFC\xFD\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fe() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFE as u64);

        assert_eq!(&buf[..], b"\xFC\xFE\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_ff() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_uint_lenenc::<LittleEndian, _>(0xFF as u64);

        assert_eq!(&buf[..], b"\xFC\xFF\x00");
    }

    #[test]
    fn it_encodes_int_u64() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64::<LittleEndian>(std::u64::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u32() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u32::<LittleEndian>(std::u32::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u24() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u24::<LittleEndian>(0xFF_FF_FF as u32);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u16() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u16::<LittleEndian>(std::u16::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u8() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u8(std::u8::MAX);

        assert_eq!(&buf[..], b"\xFF");
    }

    #[test]
    fn it_encodes_string_lenenc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_str_lenenc::<LittleEndian>("random_string");

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }

    #[test]
    fn it_encodes_string_fix() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_str("random_string");

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_string_null() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_str_nul("random_string");

        assert_eq!(&buf[..], b"random_string\0");
    }

    #[test]
    fn it_encodes_byte_lenenc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_bytes_lenenc::<LittleEndian>(b"random_string");

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }
}
