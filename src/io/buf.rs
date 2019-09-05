use byteorder::ByteOrder;
use memchr::memchr;
use std::{io, str};

pub trait Buf {
    fn advance(&mut self, cnt: usize);

    fn get_u8(&mut self) -> io::Result<u8>;

    fn get_u16<T: ByteOrder>(&mut self) -> io::Result<u16>;

    fn get_u24<T: ByteOrder>(&mut self) -> io::Result<u32>;

    fn get_i32<T: ByteOrder>(&mut self) -> io::Result<i32>;

    fn get_u32<T: ByteOrder>(&mut self) -> io::Result<u32>;

    fn get_u64<T: ByteOrder>(&mut self) -> io::Result<u64>;

    fn get_str(&mut self, len: usize) -> io::Result<&str>;

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

    fn get_u16<T: ByteOrder>(&mut self) -> io::Result<u16> {
        let val = T::read_u16(*self);
        self.advance(2);

        Ok(val)
    }

    fn get_i32<T: ByteOrder>(&mut self) -> io::Result<i32> {
        let val = T::read_i32(*self);
        self.advance(4);

        Ok(val)
    }

    fn get_u24<T: ByteOrder>(&mut self) -> io::Result<u32> {
        let val = T::read_u24(*self);
        self.advance(3);

        Ok(val)
    }

    fn get_u32<T: ByteOrder>(&mut self) -> io::Result<u32> {
        let val = T::read_u32(*self);
        self.advance(4);

        Ok(val)
    }

    fn get_u64<T: ByteOrder>(&mut self) -> io::Result<u64> {
        let val = T::read_u64(*self);
        self.advance(8);

        Ok(val)
    }

    fn get_str(&mut self, len: usize) -> io::Result<&str> {
        let buf = &self[..len];

        self.advance(len);

        if cfg!(debug_asserts) {
            str::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        } else {
            Ok(unsafe { str::from_utf8_unchecked(buf) })
        }
    }

    fn get_str_null(&mut self) -> io::Result<&str> {
        let len = memchr(b'\0', &*self).ok_or(io::ErrorKind::InvalidData)?;
        let s = &self.get_str(len + 1)?[..len];

        Ok(s)
    }
}
