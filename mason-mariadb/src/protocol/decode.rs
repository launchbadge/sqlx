// Deserializing bytes and string do the same thing. Except that string also has a null terminated deserialzer
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use failure::{err_msg, Error};

pub struct Decoder<'a> {
    pub buf: &'a Bytes,
    pub index: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a Bytes) -> Self {
        Decoder { buf, index: 0 }
    }

    #[inline]
    pub fn decode_length(&mut self) -> Result<u32, Error> {
        let length = self.decode_int_3();

        if self.buf.len() < length as usize {
            return Err(err_msg("Lengths to do not match"));
        }

        Ok(length)
    }

    #[inline]
    pub fn skip_bytes(&mut self, amount: usize) {
        self.index += amount;
    }

    #[inline]
    pub fn eof(&self) -> bool {
        self.buf.len() == self.index
    }

    #[inline]
    pub fn decode_int_lenenc(&mut self) -> Option<usize> {
        match self.buf[self.index] {
            0xFB => {
                self.index += 1;
                None
            }
            0xFC => {
                let value = Some(LittleEndian::read_u16(&self.buf[self.index + 1..]) as usize);
                self.index += 3;
                value
            }
            0xFD => {
                let value = Some(LittleEndian::read_u24(&self.buf[self.index + 1..]) as usize);
                self.index += 4;
                value
            }
            0xFE => {
                let value = Some(LittleEndian::read_u64(&self.buf[self.index + 1..]) as usize);
                self.index += 9;
                value
            }
            0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
            _ => {
                let value = Some(self.buf[self.index] as usize);
                self.index += 1;
                value
            }
        }
    }

    #[inline]
    pub fn decode_int_8(&mut self) -> u64 {
        let value = LittleEndian::read_u64(&self.buf[self.index..]);
        self.index += 8;
        value
    }

    #[inline]
    pub fn decode_int_4(&mut self) -> u32 {
        let value = LittleEndian::read_u32(&self.buf[self.index..]);
        self.index += 4;
        value
    }

    #[inline]
    pub fn decode_int_3(&mut self) -> u32 {
        let value = LittleEndian::read_u24(&self.buf[self.index..]);
        self.index += 3;
        value
    }

    #[inline]
    pub fn decode_int_2(&mut self) -> u16 {
        let value = LittleEndian::read_u16(&self.buf[self.index..]);
        self.index += 2;
        value
    }

    #[inline]
    pub fn decode_int_1(&mut self) -> u8 {
        let value = self.buf[self.index];
        self.index += 1;
        value
    }

    #[inline]
    pub fn decode_string_lenenc(&mut self) -> Bytes {
        let length = self.decode_int_3();
        let value = Bytes::from(&self.buf[self.index..self.index + length as usize]);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_string_fix(&mut self, length: u32) -> Bytes {
        let value = Bytes::from(&self.buf[self.index..self.index + length as usize]);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_string_eof(&mut self) -> Bytes {
        let value = Bytes::from(&self.buf[self.index..]);
        self.index = self.buf.len();
        value
    }

    #[inline]
    pub fn decode_string_null(&mut self) -> Result<Bytes, Error> {
        if let Some(null_index) = memchr::memchr(0, &self.buf[self.index..]) {
            let value = Bytes::from(&self.buf[self.index..self.index + null_index]);
            self.index = self.index + null_index + 1;
            Ok(value)
        } else {
            Err(err_msg("Null index no found"))
        }
    }

    #[inline]
    pub fn decode_byte_fix(&mut self, length: u32) -> Bytes {
        let value = Bytes::from(&self.buf[self.index..self.index + length as usize]);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_byte_lenenc(&mut self) -> Bytes {
        let length = self.decode_int_3();
        let value = Bytes::from(&self.buf[self.index..self.index + length as usize]);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_byte_eof(&mut self) -> Bytes {
        let value = Bytes::from(&self.buf[self.index..]);
        self.index = self.buf.len();
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};
    use failure::Error;

    // [X] it_decodes_int_lenenc
    // [X] it_decodes_int_8
    // [X] it_decodes_int_4
    // [X] it_decodes_int_3
    // [X] it_decodes_int_2
    // [X] it_decodes_int_1
    // [X] it_decodes_string_lenenc
    // [X] it_decodes_string_fix
    // [X] it_decodes_string_eof
    // [X] it_decodes_string_null
    // [X] it_decodes_byte_lenenc
    // [X] it_decodes_byte_eof

    #[test]
    fn it_decodes_int_lenenc_0x_fb() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\xFB".to_vec()).freeze());
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, None);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fc() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\xFC\x01\x01".to_vec()).freeze());
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(257));
        assert_eq!(index, 3);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fd() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\xFD\x01\x01\x01".to_vec()).freeze());
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(65793));
        assert_eq!(index, 4);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fe() {
        let mut decoder = Decoder::new(
            &BytesMut::from(b"\xFE\x01\x01\x01\x01\x01\x01\x01\x01".to_vec()).freeze(),
        );
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(72340172838076673));
        assert_eq!(index, 9);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fa() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\xFA".to_vec()).freeze());
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(0xfA));
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_int_8() {
        let mut decoder =
            Decoder::new(&BytesMut::from(b"\x01\x01\x01\x01\x01\x01\x01\x01".to_vec()).freeze());
        let int: u64 = decoder.decode_int_8();

        assert_eq!(int, 72340172838076673);
        assert_eq!(index, 8);
    }

    #[test]
    fn it_decodes_int_4() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01\x01\x01\x01".to_vec()).freeze());
        let int: u32 = decoder.decode_int_4();

        assert_eq!(int, 16843009);
        assert_eq!(index, 4);
    }

    #[test]
    fn it_decodes_int_3() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01\x01\x01".to_vec()).freeze());
        let int: u32 = decoder.decode_int_3();

        assert_eq!(int, 65793);
        assert_eq!(index, 3);
    }

    #[test]
    fn it_decodes_int_2() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01\x01".to_vec()).freeze());
        let int: u16 = decoder.decode_int_2();

        assert_eq!(int, 257);
        assert_eq!(index, 2);
    }

    #[test]
    fn it_decodes_int_1() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01".to_vec()).freeze());
        let int: u8 = decoder.decode_int_1();

        assert_eq!(int, 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_string_lenenc() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01\x00\x00\x01".to_vec()).freeze());
        let string: Bytes = decoder.decode_string_lenenc();

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 4);
    }

    #[test]
    fn it_decodes_string_fix() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01".to_vec()).freeze());
        let string: Bytes = decoder.decode_string_fix(1);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_string_eof() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01".to_vec()).freeze());
        let string: Bytes = decoder.decode_string_eof();

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_string_null() -> Result<(), Error> {
        let mut decoder = Decoder::new(&BytesMut::from(b"random\x00\x01".to_vec()).freeze());
        let string: Bytes = decoder.decode_string_null()?;

        assert_eq!(&string[..], b"random");

        assert_eq!(string.len(), 6);
        // Skips null byte
        assert_eq!(index, 7);

        Ok(())
    }

    #[test]
    fn it_decodes_byte_fix() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01".to_vec()).freeze());
        let string: Bytes = decoder.decode_byte_fix(1);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_byte_eof() {
        let mut decoder = Decoder::new(&BytesMut::from(b"\x01".to_vec()).freeze());
        let string: Bytes = decoder.decode_byte_eof();

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }
}
