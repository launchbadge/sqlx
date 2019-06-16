use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;

#[inline]
pub fn deserialize_int_lenenc(buf: &Vec<u8>, index: &mut usize) -> Option<usize> {
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
            let value =
                Some((buf[*index + 1] + buf[*index + 2] << 8 + buf[*index + 3] << 16) as usize);
            *index += 4;
            value
        }
        0xFE => {
            let value = Some(LittleEndian::read_u64(&buf[*index..]) as usize);
            *index += 9;
            value
        }
        0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
        _ => {
            let value = Some(buf[*index] as usize);
            *index += 2;
            value
        }
    }
}

#[inline]
pub fn deserialize_int_8(buf: &Vec<u8>, index: &mut usize) -> u64 {
    let value = LittleEndian::read_u64(&buf[*index..]);
    *index += 8;
    value
}

#[inline]
pub fn deserialize_int_4(buf: &Vec<u8>, index: &mut usize) -> u32 {
    let value = LittleEndian::read_u32(&buf[*index..]);
    *index += 4;
    value
}

#[inline]
pub fn deserialize_int_3(buf: &Vec<u8>, index: &mut usize) -> u32 {
    let value = LittleEndian::read_u24(&buf[*index..]);
    *index += 3;
    value
}

#[inline]
pub fn deserialize_int_2(buf: &Vec<u8>, index: &mut usize) -> u16 {
    let value = LittleEndian::read_u16(&buf[*index..]);
    *index += 2;
    value
}

#[inline]
pub fn deserialize_int_1(buf: &Vec<u8>, index: &mut usize) -> u8 {
    let value = buf[*index];
    *index += 1;
    value
}

#[inline]
pub fn deserialize_string_lenenc(buf: &Vec<u8>, index: &mut usize) -> Bytes {
    let length = deserialize_int_3(&buf, &mut *index);
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_string_fix(buf: &Vec<u8>, index: &mut usize, length: usize) -> Bytes {
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_string_eof(buf: &Vec<u8>, index: &mut usize) -> Bytes {
    let value = Bytes::from(&buf[*index..]);
    *index = buf.len();
    value
}

#[inline]
pub fn deserialize_string_null(buf: &Vec<u8>, index: &mut usize) -> Bytes {
    let null_index = memchr::memchr(b'\0', &buf[*index..]).unwrap();
    let value = Bytes::from(&buf[*index..null_index]);
    *index = null_index + 1;
    value
}

#[inline]
pub fn deserialize_byte_fix(buf: &Vec<u8>, index: &mut usize, length: usize) -> Bytes {
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_byte_lenenc(buf: &Vec<u8>, index: &mut usize) -> Bytes {
    let length = deserialize_int_3(&buf, &mut *index);
    let value = Bytes::from(&buf[*index..*index + length as usize]);
    *index = *index + length as usize;
    value
}

#[inline]
pub fn deserialize_byte_eof(buf: &Vec<u8>, index: &mut usize) -> Bytes {
    let value = Bytes::from(&buf[*index..]);
    *index = buf.len();
    value
}
