use byteorder::ByteOrder;
use memchr::memchr;
use std::{convert::TryInto, io, mem::size_of, str};

pub trait Buf {
    fn advance(&mut self, cnt: usize);

    fn get_u8(&mut self) -> io::Result<u8>;

    fn get_u16<T: ByteOrder>(&mut self) -> io::Result<u16>;

    fn get_u24<T: ByteOrder>(&mut self) -> io::Result<u32>;

    fn get_i32<T: ByteOrder>(&mut self) -> io::Result<i32>;

    fn get_u32<T: ByteOrder>(&mut self) -> io::Result<u32>;

    fn get_u64<T: ByteOrder>(&mut self) -> io::Result<u64>;

    // TODO?: Move to mariadb::io::BufExt
    fn get_uint<T: ByteOrder>(&mut self, n: usize) -> io::Result<u64>;

    // TODO?: Move to mariadb::io::BufExt
    fn get_uint_lenenc<T: ByteOrder>(&mut self) -> io::Result<u64>;

    fn get_str(&mut self, len: usize) -> io::Result<&str>;

    // TODO?: Move to mariadb::io::BufExt
    fn get_str_eof(&mut self) -> io::Result<&str>;

    fn get_str_nul(&mut self) -> io::Result<&str>;

    // TODO?: Move to mariadb::io::BufExt
    fn get_str_lenenc<T: ByteOrder>(&mut self) -> io::Result<&str>;
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

    fn get_uint<T: ByteOrder>(&mut self, n: usize) -> io::Result<u64> {
        let val = T::read_uint(*self, n);
        self.advance(n);

        Ok(val)
    }

    fn get_uint_lenenc<T: ByteOrder>(&mut self) -> io::Result<u64> {
        Ok(match self.get_u8()? {
            0xFC => self.get_u16::<T>()? as u64,
            0xFD => self.get_u24::<T>()? as u64,
            0xFE => self.get_u64::<T>()? as u64,
            // ? 0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
            value => value as u64,
        })
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

    fn get_str_eof(&mut self) -> io::Result<&str> {
        self.get_str(self.len())
    }

    fn get_str_nul(&mut self) -> io::Result<&str> {
        let len = memchr(b'\0', &*self).ok_or(io::ErrorKind::InvalidData)?;
        let s = &self.get_str(len + 1)?[..len];

        Ok(s)
    }

    fn get_str_lenenc<T: ByteOrder>(&mut self) -> io::Result<&str> {
        let len = self.get_uint_lenenc::<T>()?;
        let s = self.get_str(len as usize)?;

        Ok(s)
    }
}
