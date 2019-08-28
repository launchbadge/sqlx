use byteorder::ByteOrder;
use memchr::memchr;
use std::{io, mem::size_of, str, u16, u32, u8};

pub trait BufMut {
    fn advance(&mut self, cnt: usize);

    fn put_u8(&mut self, val: u8);

    fn put_u16<T: ByteOrder>(&mut self, val: u16);
    
    fn put_i16<T: ByteOrder>(&mut self, val: i16);

    fn put_u24<T: ByteOrder>(&mut self, val: u32);
    
    fn put_i32<T: ByteOrder>(&mut self, val: i32);

    fn put_u32<T: ByteOrder>(&mut self, val: u32);

    fn put_u64<T: ByteOrder>(&mut self, val: u64);

    // TODO: Move to mariadb::io::BufMutExt
    fn put_u64_lenenc<T: ByteOrder>(&mut self, val: u64);

    fn put_str_nul(&mut self, val: &str);

    // TODO: Move to mariadb::io::BufMutExt
    fn put_str_lenenc<T: ByteOrder>(&mut self, val: &str);

    // TODO: Move to mariadb::io::BufMutExt
    fn put_str_eof(&mut self, val: &str);
}

impl BufMut for Vec<u8> {
    fn advance(&mut self, cnt: usize) {
        self.resize(self.len() + cnt, 0);
    }

    fn put_u8(&mut self, val: u8) {
        self.push(val);
    }

    fn put_i16<T: ByteOrder>(&mut self, val: i16) {
        let mut buf = [0; 4];
        T::write_i16(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_u16<T: ByteOrder>(&mut self, val: u16) {
        let mut buf = [0; 2];
        T::write_u16(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_u24<T: ByteOrder>(&mut self, val: u32) {
        let mut buf = [0; 3];
        T::write_u24(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_i32<T: ByteOrder>(&mut self, val: i32) {
        let mut buf = [0; 4];
        T::write_i32(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_u32<T: ByteOrder>(&mut self, val: u32) {
        let mut buf = [0; 4];
        T::write_u32(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_u64<T: ByteOrder>(&mut self, val: u64) {
        let mut buf = [0; 8];
        T::write_u64(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_u64_lenenc<T: ByteOrder>(&mut self, value: u64) {
        // https://mariadb.com/kb/en/library/protocol-data-types/#length-encoded-integers
        if value > 0xFF_FF_FF {
            // Integer value is encoded in the next 8 bytes (9 bytes total)
            self.push(0xFE);
            self.put_u64::<T>(value);
        } else if value > u16::MAX as _ {
            // Integer value is encoded in the next 3 bytes (4 bytes total)
            self.push(0xFD);
            self.put_u24::<T>(value as u32);
        } else if value > u8::MAX as _ {
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
    }

    fn put_str_eof(&mut self, val: &str) {
        self.extend_from_slice(val.as_bytes());
    }

    fn put_str_nul(&mut self, val: &str) {
        self.extend_from_slice(val.as_bytes());
        self.push(0);
    }

    fn put_str_lenenc<T: ByteOrder>(&mut self, val: &str) {
        self.put_u64_lenenc::<T>(val.len() as u64);
        self.extend_from_slice(val.as_bytes());
    }
}
