use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use failure::{err_msg, Error};

// Deserializing bytes and string do the same thing. Except that string also has a null terminated deserialzer
use super::packets::packet_header::PacketHeader;

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

        if self.buf.len() - self.index < length as usize {
            return Err(err_msg("Lengths to do not match when decoding length"));
        }

        Ok(length)
    }

    #[inline]
    pub fn peek_tag(&self) -> Option<&u8> {
        if self.buf.len() < self.index + 4 {
            None
        } else {
            Some(&self.buf[self.index + 4])
        }
    }

    #[inline]
    pub fn peek_packet_header(&self) -> Result<PacketHeader, Error> {
        let length: u32 = (self.buf[self.index] as u32) + ((self.buf[self.index + 1] as u32) << 8) + ((self.buf[self.index + 2] as u32) << 16);
        let seq_no = self.buf[self.index + 3];

        if self.buf.len() - self.index < length as usize {
            return Err(err_msg("Lengths to do not match when peeking header"));
        }

        Ok(PacketHeader { length, seq_no })
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
    pub fn eof_byte(&self) -> bool {
        self.buf[self.index] == 0xFE
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
        let length = self.decode_int_1();
        let value = self.buf.slice(self.index, self.index + length as usize);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_string_fix(&mut self, length: u32) -> Bytes {
        let value = self.buf.slice(self.index, self.index + length as usize);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_string_eof(&mut self, length: Option<usize>) -> Bytes {
        let value = self.buf.slice(self.index, if let Some(len) = length {
            if len >= self.index {
                len
            } else {
                self.buf.len()
            }
        } else {
            self.buf.len()
        });
        self.index = self.buf.len();
        value
    }

    #[inline]
    pub fn decode_string_null(&mut self) -> Result<Bytes, Error> {
        if let Some(null_index) = memchr::memchr(0, &self.buf[self.index..]) {
            let value = self.buf.slice(self.index, self.index + null_index);
            self.index = self.index + null_index + 1;
            Ok(value)
        } else {
            Err(err_msg("Null index no found"))
        }
    }

    #[inline]
    pub fn decode_byte_fix(&mut self, length: u32) -> Bytes {
        let value = self.buf.slice(self.index, self.index + length as usize);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_byte_lenenc(&mut self) -> Bytes {
        let length = self.decode_int_1();
        let value = self.buf.slice(self.index, self.index + length as usize);
        self.index = self.index + length as usize;
        value
    }

    #[inline]
    pub fn decode_byte_eof(&mut self, length: Option<usize>) -> Bytes {
        let value = self.buf.slice(self.index, if let Some(len) = length {
            if len >= self.index {
                len
            } else {
                self.buf.len()
            }
        } else {
            self.buf.len()
        });
        self.index = self.buf.len();
        value
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use failure::Error;
    use crate::__bytes_builder;

    use super::*;

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
        let buf = __bytes_builder!(0xFB_u8);
        let mut decoder = Decoder::new(&buf);
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, None);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fc() {
        let buf =__bytes_builder!(0xFCu8, 1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(0x0101));
        assert_eq!(decoder.index, 3);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fd() {
        let buf = __bytes_builder!(0xFDu8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(0x010101));
        assert_eq!(decoder.index, 4);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fe() {
        let buf = __bytes_builder!(0xFE_u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(0x0101010101010101));
        assert_eq!(decoder.index, 9);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fa() {
        let buf = __bytes_builder!(0xFA_u8);
        let mut decoder = Decoder::new(&buf);
        let int: Option<usize> = decoder.decode_int_lenenc();

        assert_eq!(int, Some(0xFA));
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_int_8() {
        let buf = __bytes_builder!(1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: u64 = decoder.decode_int_8();

        assert_eq!(int, 0x0101010101010101);
        assert_eq!(decoder.index, 8);
    }

    #[test]
    fn it_decodes_int_4() {
        let buf = __bytes_builder!(1u8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: u32 = decoder.decode_int_4();

        assert_eq!(int, 0x01010101);
        assert_eq!(decoder.index, 4);
    }

    #[test]
    fn it_decodes_int_3() {
        let buf = __bytes_builder!(1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: u32 = decoder.decode_int_3();

        assert_eq!(int, 0x010101);
        assert_eq!(decoder.index, 3);
    }

    #[test]
    fn it_decodes_int_2() {
        let buf = __bytes_builder!(1u8, 1u8);
        let mut decoder = Decoder::new(&buf);
        let int: u16 = decoder.decode_int_2();

        assert_eq!(int, 0x0101);
        assert_eq!(decoder.index, 2);
    }

    #[test]
    fn it_decodes_int_1() {
        let buf = __bytes_builder!(1u8);
        let mut decoder = Decoder::new(&buf);
        let int: u8 = decoder.decode_int_1();

        assert_eq!(int, 1u8);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_string_lenenc() {
        let buf = __bytes_builder!(3u8, b"sup");
        let mut decoder = Decoder::new(&buf);
        let string: Bytes = decoder.decode_string_lenenc();

        assert_eq!(string[..], b"sup"[..]);
        assert_eq!(string.len(), 3);
        assert_eq!(decoder.index, 4);
    }

    #[test]
    fn it_decodes_string_fix() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(&buf);
        let string: Bytes = decoder.decode_string_fix(1);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_string_eof() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(&buf);
        let string: Bytes = decoder.decode_string_eof(None);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_string_null() -> Result<(), Error> {
        let buf = __bytes_builder!(b"random\0", 1u8);
        let mut decoder = Decoder::new(&buf);
        let string: Bytes = decoder.decode_string_null()?;

        assert_eq!(&string[..], b"random");

        assert_eq!(string.len(), 6);
        // Skips null byte
        assert_eq!(decoder.index, 7);

        Ok(())
    }

    #[test]
    fn it_decodes_byte_fix() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(&buf);
        let string: Bytes = decoder.decode_byte_fix(1);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_byte_eof() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(&buf);
        let string: Bytes = decoder.decode_byte_eof(None);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }
}
