use std::io;

use byteorder::ByteOrder;

use crate::io::Buf;

pub trait BufExt {
    fn get_uint_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<u64>>;

    fn get_str_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&str>>;

    fn get_bytes_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&[u8]>>;
}

impl BufExt for &'_ [u8] {
    fn get_uint_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<u64>> {
        Ok(match self.get_u8()? {
            0xFB => None,
            0xFC => Some(u64::from(self.get_u16::<T>()?)),
            0xFD => Some(u64::from(self.get_u24::<T>()?)),
            0xFE => Some(self.get_u64::<T>()?),

            value => Some(u64::from(value)),
        })
    }

    fn get_str_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&str>> {
        self.get_uint_lenenc::<T>()?
            .map(move |len| self.get_str(len as usize))
            .transpose()
    }

    fn get_bytes_lenenc<T: ByteOrder>(&mut self) -> io::Result<Option<&[u8]>> {
        self.get_uint_lenenc::<T>()?
            .map(move |len| self.get_bytes(len as usize))
            .transpose()
    }
}
