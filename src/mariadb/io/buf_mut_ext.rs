use byteorder::ByteOrder;
use crate::io::BufMut;
use std::{u8, u16, u32, u64};

pub trait BufMutExt {
    fn put_u64_lenenc<T: ByteOrder>(&mut self, val: Option<u64>);

    fn put_str_lenenc<T: ByteOrder>(&mut self, val: &str);

    fn put_str(&mut self, val: &str);

    fn put_byte_lenenc<T: ByteOrder>(&mut self, val: &[u8]);
}

impl BufMutExt for Vec<u8> {
    fn put_u64_lenenc<T: ByteOrder>(&mut self, value: Option<u64>) {
        if let Some(value) = value {
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

    fn put_str(&mut self, val: &str) {
        self.extend_from_slice(val.as_bytes());
    }

    fn put_str_lenenc<T: ByteOrder>(&mut self, val: &str) {
        self.put_u64_lenenc::<T>(Some(val.len() as u64));
        self.extend_from_slice(val.as_bytes());
    }

    fn put_byte_lenenc<T: ByteOrder>(&mut self, val: &[u8]) {
        self.put_u64_lenenc::<T>(Some(val.len() as u64));
        self.extend_from_slice(val);
    }
}
