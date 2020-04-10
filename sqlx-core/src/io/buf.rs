use byteorder::ByteOrder;
use memchr::memchr;
use std::{io, slice, str};

pub trait Buf<'a> {
    fn advance(&mut self, cnt: usize);

    fn get_uint<T: ByteOrder>(&mut self, n: usize) -> io::Result<u64>;

    fn get_i8(&mut self) -> io::Result<i8>;

    fn get_u8(&mut self) -> io::Result<u8>;

    fn get_u16<T: ByteOrder>(&mut self) -> io::Result<u16>;

    fn get_i16<T: ByteOrder>(&mut self) -> io::Result<i16>;

    fn get_u24<T: ByteOrder>(&mut self) -> io::Result<u32>;

    fn get_i32<T: ByteOrder>(&mut self) -> io::Result<i32>;

    fn get_i64<T: ByteOrder>(&mut self) -> io::Result<i64>;

    fn get_u32<T: ByteOrder>(&mut self) -> io::Result<u32>;

    fn get_f32<T: ByteOrder>(&mut self) -> io::Result<f32>;

    fn get_u64<T: ByteOrder>(&mut self) -> io::Result<u64>;

    fn get_f64<T: ByteOrder>(&mut self) -> io::Result<f64>;

    fn get_str(&mut self, len: usize) -> io::Result<&'a str>;

    fn get_str_nul(&mut self) -> io::Result<&'a str>;

    fn get_bytes(&mut self, len: usize) -> io::Result<&'a [u8]>;
}

impl<'a> Buf<'a> for &'a [u8] {
    fn advance(&mut self, cnt: usize) {
        *self = &self[cnt..];
    }

    fn get_uint<T: ByteOrder>(&mut self, n: usize) -> io::Result<u64> {
        let val = T::read_uint(*self, n);
        self.advance(n);

        Ok(val)
    }

    fn get_i8(&mut self) -> io::Result<i8> {
        let val = self[0];
        self.advance(1);

        Ok(val as i8)
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

    fn get_i16<T: ByteOrder>(&mut self) -> io::Result<i16> {
        let val = T::read_i16(*self);
        self.advance(2);

        Ok(val)
    }

    fn get_u24<T: ByteOrder>(&mut self) -> io::Result<u32> {
        let val = T::read_u24(*self);
        self.advance(3);

        Ok(val)
    }

    fn get_i32<T: ByteOrder>(&mut self) -> io::Result<i32> {
        let val = T::read_i32(*self);
        self.advance(4);

        Ok(val)
    }

    fn get_i64<T: ByteOrder>(&mut self) -> io::Result<i64> {
        let val = T::read_i64(*self);
        self.advance(4);

        Ok(val)
    }

    fn get_u32<T: ByteOrder>(&mut self) -> io::Result<u32> {
        let val = T::read_u32(*self);
        self.advance(4);

        Ok(val)
    }

    fn get_f32<T: ByteOrder>(&mut self) -> io::Result<f32> {
        let val = T::read_f32(*self);
        self.advance(4);

        Ok(val)
    }

    fn get_u64<T: ByteOrder>(&mut self) -> io::Result<u64> {
        let val = T::read_u64(*self);
        self.advance(8);

        Ok(val)
    }

    fn get_f64<T: ByteOrder>(&mut self) -> io::Result<f64> {
        let val = T::read_f64(*self);
        self.advance(8);

        Ok(val)
    }

    fn get_str(&mut self, len: usize) -> io::Result<&'a str> {
        str::from_utf8(self.get_bytes(len)?)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    fn get_str_nul(&mut self) -> io::Result<&'a str> {
        let len = memchr(b'\0', &*self).ok_or(io::ErrorKind::InvalidData)?;
        let s = &self.get_str(len + 1)?[..len];

        Ok(s)
    }

    fn get_bytes(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let buf = &self[..len];
        self.advance(len);

        Ok(buf)
    }
}

pub trait ToBuf {
    fn to_buf(&self) -> &[u8];
}

impl ToBuf for [u8] {
    fn to_buf(&self) -> &[u8] {
        self
    }
}

impl ToBuf for u8 {
    fn to_buf(&self) -> &[u8] {
        slice::from_ref(self)
    }
}
