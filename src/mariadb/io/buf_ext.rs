use crate::io::Buf;
use byteorder::ByteOrder;
use std::io;

pub trait BufExt {
    fn get_uint<T: ByteOrder>(&mut self, n: usize) -> io::Result<u64>;
    fn get_uint_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<u64>>;
    fn get_str_eof(&mut self) -> io::Result<&str>;
    fn get_str_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&str>>;
    fn get_byte_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&[u8]>>;
}

impl<'a> BufExt for &'a [u8] {
    fn get_uint<T: ByteOrder>(&mut self, n: usize) -> io::Result<u64> {
        let val = T::read_uint(*self, n);
        self.advance(n);

        Ok(val)
    }

    fn get_uint_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<u64>> {
        Ok(match self.get_u8()? {
            0xFB => None,
            0xFC => Some(u64::from(self.get_u16::<T>()?)),
            0xFD => Some(u64::from(self.get_u24::<T>()?)),
            0xFE => Some(self.get_u64::<T>()?),
            // ? 0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
            value => Some(u64::from(value)),
        })
    }

    fn get_str_eof(&mut self) -> io::Result<&str> {
        self.get_str(self.len())
    }

    fn get_str_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&str>> {
        self.get_uint_lenenc::<T>()?
            .map(move |len| self.get_str(len as usize))
            .transpose()
    }

    fn get_byte_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&[u8]>> {
        Ok(self
            .get_uint_lenenc::<T>()?
            .map(|len| &self[..len as usize]))
    }
}
