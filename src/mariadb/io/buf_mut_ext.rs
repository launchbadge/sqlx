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
