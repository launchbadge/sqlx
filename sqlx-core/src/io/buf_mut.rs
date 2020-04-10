use byteorder::ByteOrder;
use std::{str, u16, u32, u8};

pub trait BufMut {
    fn advance(&mut self, cnt: usize);

    fn put_u8(&mut self, val: u8);

    fn put_u16<T: ByteOrder>(&mut self, val: u16);

    fn put_i16<T: ByteOrder>(&mut self, val: i16);

    fn put_u24<T: ByteOrder>(&mut self, val: u32);

    fn put_i32<T: ByteOrder>(&mut self, val: i32);

    fn put_u32<T: ByteOrder>(&mut self, val: u32);

    fn put_f32<T: ByteOrder>(&mut self, val: f32);

    fn put_u64<T: ByteOrder>(&mut self, val: u64);

    fn put_f64<T: ByteOrder>(&mut self, val: f64);

    fn put_bytes(&mut self, val: &[u8]);

    fn put_str(&mut self, val: &str);

    fn put_str_nul(&mut self, val: &str);
}

impl BufMut for Vec<u8> {
    fn advance(&mut self, cnt: usize) {
        self.resize(self.len() + cnt, 0);
    }

    fn put_u8(&mut self, val: u8) {
        self.push(val);
    }

    fn put_u16<T: ByteOrder>(&mut self, val: u16) {
        let mut buf = [0; 2];
        T::write_u16(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_i16<T: ByteOrder>(&mut self, val: i16) {
        let mut buf = [0; 2];
        T::write_i16(&mut buf, val);
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

    fn put_f32<T: ByteOrder>(&mut self, val: f32) {
        let mut buf = [0; 4];
        T::write_f32(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_u64<T: ByteOrder>(&mut self, val: u64) {
        let mut buf = [0; 8];
        T::write_u64(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_f64<T: ByteOrder>(&mut self, val: f64) {
        let mut buf = [0; 8];
        T::write_f64(&mut buf, val);
        self.extend_from_slice(&buf);
    }

    fn put_bytes(&mut self, val: &[u8]) {
        self.extend_from_slice(val);
    }

    fn put_str(&mut self, val: &str) {
        self.extend_from_slice(val.as_bytes());
    }

    fn put_str_nul(&mut self, val: &str) {
        self.put_str(val);
        self.push(0);
    }
}
