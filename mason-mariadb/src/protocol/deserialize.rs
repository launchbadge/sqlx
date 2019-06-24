// Deserializing bytes and string do the same thing. Except that string also has a null terminated deserialzer
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use failure::Error;
use failure::err_msg;

#[inline]
pub fn deserialize_length(buf: &Bytes, index: &mut usize) -> Result<u32, Error> {
    let length = deserialize_int_3(&buf, index);

    if buf.len() < length as usize {
        return Err(err_msg("Lengths to do not match"));
    }

    Ok(length)
}

#[inline]
pub fn deserialize_int_lenenc(buf: &Bytes, index: &mut usize) -> Option<usize> {
    match buf[*index] {
        0xFB => {
            *index += 1;
            None
        }
        0xFC => {
            let value = Some(LittleEndian::read_u16(&buf[*index + 1..]) as usize);
            *index += 3;
            value
        }
        0xFD => {
            let value = Some(LittleEndian::read_u24(&buf[*index + 1..]) as usize);
            *index += 4;
            value
        }
        0xFE => {
            let value = Some(LittleEndian::read_u64(&buf[*index + 1..]) as usize);
            *index += 9;
            value
        }
        0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
        _ => {
            let value = Some(buf[*index + 1] as usize);
            *index += 2;
            value
        }
    }
}

#[inline]
pub fn deserialize_int_8(buf: &Bytes, index: &mut usize) -> u64 {
    let value = LittleEndian::read_u64(&buf[*index..]);
    *index += 8;
    value
}

#[inline]
pub fn deserialize_int_4(buf: &Bytes, index: &mut usize) -> u32 {
    let value = LittleEndian::read_u32(&buf[*index..]);
    *index += 4;
    value
}

#[inline]
pub fn deserialize_int_3(buf: &Bytes, index: &mut usize) -> u32 {
    let value = LittleEndian::read_u24(&buf[*index..]);
    *index += 3;
    value
}

#[inline]
pub fn deserialize_int_2(buf: &Bytes, index: &mut usize) -> u16 {
    let value = LittleEndian::read_u16(&buf[*index..]);
    *index += 2;
    value
}

#[inline]
pub fn deserialize_int_1(buf: &Bytes, index: &mut usize) -> u8 {
    let value = buf[*index];
    *index += 1;
    value
}

#[inline]
pub fn deserialize_string_lenenc(buf: &Bytes, index: &mut usize) -> Bytes {
    let length = deserialize_int_3(&buf, &mut *index);
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_string_fix(buf: &Bytes, index: &mut usize, length: usize) -> Bytes {
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_string_eof(buf: &Bytes, index: &mut usize) -> Bytes {
    let value = Bytes::from(&buf[*index..]);
    *index = buf.len();
    value
}

#[inline]
pub fn deserialize_string_null(buf: &Bytes, index: &mut usize) -> Bytes {
    let null_index = memchr::memchr(0, &buf[*index..]).unwrap();
    let value = Bytes::from(&buf[*index..*index + null_index]);
    *index = *index + null_index + 1;
    value
}

#[inline]
pub fn deserialize_byte_fix(buf: &Bytes, index: &mut usize, length: usize) -> Bytes {
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_byte_lenenc(buf: &Bytes, index: &mut usize) -> Bytes {
    let length = deserialize_int_3(&buf, &mut *index);
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_byte_eof(buf: &Bytes, index: &mut usize) -> Bytes {
    let value = Bytes::from(&buf[*index..]);
    *index = buf.len();
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};
    use std::error::Error;

    // [X] deserialize_int_lenenc
    // [X] deserialize_int_8
    // [X] deserialize_int_4
    // [X] deserialize_int_3
    // [X] deserialize_int_2
    // [X] deserialize_int_1
    // [X] deserialize_string_lenenc
    // [X] deserialize_string_fix
    // [X] deserialize_string_eof
    // [X] deserialize_string_null
    // [X] deserialize_byte_lenenc
    // [X] deserialize_byte_eof

    #[test]
    fn it_decodes_int_lenenc_0x_fb() {
        let buf: BytesMut  = BytesMut::from(b"\xFB".to_vec());
        let mut index = 0;
        let int: Option<usize> = deserialize_int_lenenc(&buf.freeze(), &mut index);

        assert_eq!(int, None);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fc() {
        let buf = BytesMut::from(b"\xFC\x01\x01".to_vec());
        let mut index = 0;
        let int: Option<usize> = deserialize_int_lenenc(&buf.freeze(), &mut index);

        assert_eq!(int, Some(257));
        assert_eq!(index, 3);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fd() {
        let buf = BytesMut::from(b"\xFD\x01\x01\x01".to_vec());
        let mut index = 0;
        let int: Option<usize> = deserialize_int_lenenc(&buf.freeze(), &mut index);

        assert_eq!(int, Some(65793));
        assert_eq!(index, 4);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fe() {
        let buf = BytesMut::from(b"\xFE\x01\x01\x01\x01\x01\x01\x01\x01".to_vec());
        let mut index = 0;
        let int: Option<usize> = deserialize_int_lenenc(&buf.freeze(), &mut index);

        assert_eq!(int, Some(72340172838076673));
        assert_eq!(index, 9);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fa() {
        let buf = BytesMut::from(b"\xFA\x01".to_vec());
        let mut index = 0;
        let int: Option<usize> = deserialize_int_lenenc(&buf.freeze(), &mut index);

        assert_eq!(int, Some(1));
        assert_eq!(index, 2);
    }

    #[test]
    fn it_decodes_int_8() {
        let buf = BytesMut::from(b"\x01\x01\x01\x01\x01\x01\x01\x01".to_vec());
        let mut index = 0;
        let int: u64 = deserialize_int_8(&buf.freeze(), &mut index);

        assert_eq!(int, 72340172838076673);
        assert_eq!(index, 8);
    }

    #[test]
    fn it_decodes_int_4() {
        let buf = BytesMut::from(b"\x01\x01\x01\x01".to_vec());
        let mut index = 0;
        let int: u32 = deserialize_int_4(&buf.freeze(), &mut index);

        assert_eq!(int, 16843009);
        assert_eq!(index, 4);
    }

    #[test]
    fn it_decodes_int_3() {
        let buf = BytesMut::from(b"\x01\x01\x01".to_vec());
        let mut index = 0;
        let int: u32 = deserialize_int_3(&buf.freeze(), &mut index);

        assert_eq!(int, 65793);
        assert_eq!(index, 3);
    }

    #[test]
    fn it_decodes_int_2() {
        let buf = BytesMut::from(b"\x01\x01".to_vec());
        let mut index = 0;
        let int: u16 = deserialize_int_2(&buf.freeze(), &mut index);

        assert_eq!(int, 257);
        assert_eq!(index, 2);
    }

    #[test]
    fn it_decodes_int_1() {
        let buf = BytesMut::from(b"\x01".to_vec());
        let mut index = 0;
        let int: u8 = deserialize_int_1(&buf.freeze(), &mut index);

        assert_eq!(int, 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_string_lenenc() {
        let buf = BytesMut::from(b"\x01\x00\x00\x01".to_vec());
        let mut index = 0;
        let string: Bytes = deserialize_string_lenenc(&buf.freeze(), &mut index);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 4);
    }

    #[test]
    fn it_decodes_string_fix() {
        let buf = BytesMut::from(b"\x01".to_vec());
        let mut index = 0;
        let string: Bytes = deserialize_string_fix(&buf.freeze(), &mut index, 1);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_string_eof() {
        let buf = BytesMut::from(b"\x01".to_vec());
        let mut index = 0;
        let string: Bytes = deserialize_string_eof(&buf.freeze(), &mut index);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }

     #[test]
     fn it_decodes_string_null() {
         let buf = BytesMut::from(b"random\x00\x01".to_vec());
         let mut index = 0;
         let string: Bytes = deserialize_string_null(&buf.freeze(), &mut index);

         assert_eq!(string[0], b'r');
         assert_eq!(string[1], b'a');
         assert_eq!(string[2], b'n');
         assert_eq!(string[3], b'd');
         assert_eq!(string[4], b'o');
         assert_eq!(string[5], b'm');
         assert_eq!(string.len(), 6);
         // Skips null byte
         assert_eq!(index, 7);
     }

    #[test]
    fn it_decodes_byte_fix() {
        let buf = BytesMut::from(b"\x01".to_vec());
        let mut index = 0;
        let string: Bytes = deserialize_byte_fix(&buf.freeze(), &mut index, 1);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }

    #[test]
    fn it_decodes_byte_eof() {
        let buf = BytesMut::from(b"\x01".to_vec());
        let mut index = 0;
        let string: Bytes = deserialize_byte_eof(&buf.freeze(), &mut index);

        assert_eq!(string[0], b'\x01');
        assert_eq!(string.len(), 1);
        assert_eq!(index, 1);
    }
}
