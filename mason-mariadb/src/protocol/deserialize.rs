use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;

#[inline]
pub fn deserialize_int_lenenc(buf: &Vec<u8>, index: &usize) -> (Option<usize>, usize) {
    match buf[*index] {
        0xFB => (None, *index + 1),
        0xFC => (Some(LittleEndian::read_u16(&buf[*index + 1..]) as usize), *index + 2),
        0xFD => (Some((buf[*index + 1] + buf[*index + 2] << 8 + buf[*index + 3] << 16) as usize), *index + 3),
        0xFE => (Some(LittleEndian::read_u64(&buf[*index..]) as usize), *index + 8),
        0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
        _ => (Some(buf[*index] as usize), *index + 1),
    }
}

#[inline]
pub fn deserialize_int_4(buf: &Vec<u8>, index: &usize) -> (u32, usize) {
    (LittleEndian::read_u32(&buf[*index..]), index + 3)
}

#[inline]
pub fn deserialize_int_3(buf: &Vec<u8>, index: &usize) -> (u32, usize) {
    (LittleEndian::read_u24(&buf[*index..]), index + 3)
}

#[inline]
pub fn deserialize_int_2(buf: &Vec<u8>, index: &usize) -> (u16, usize) {
    (LittleEndian::read_u16(&buf[*index..]), index + 2)
}

#[inline]
pub fn deserialize_int_1(buf: &Vec<u8>, index: &usize) -> (u8, usize) {
    (buf[*index], index + 1)
}

#[inline]
pub fn deserialize_string_lenenc(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    let (length, index) = deserialize_int_3(&buf, &index);
    (Bytes::from(&buf[index..index + length as usize]), index + length as usize)
}

#[inline]
pub fn deserialize_string_fix(buf: &Vec<u8>, index: &usize, length: usize) -> (Bytes, usize) {
    (Bytes::from(&buf[*index..index + length as usize]), index + length as usize)
}

#[inline]
pub fn deserialize_string_eof(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    (Bytes::from(&buf[*index..]), buf.len())
}

#[inline]
pub fn deserialize_string_null(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    let null_index = memchr::memchr(b'\0', &buf[*index..]).unwrap();
    (Bytes::from(&buf[*index..null_index]), null_index + 1)
}

#[inline]
pub fn deserialize_byte_fix(buf: &Vec<u8>, index: &usize, length: usize) -> (Bytes, usize) {
    (Bytes::from(&buf[*index..index + length as usize]), index + length as usize)
}

#[inline]
pub fn deserialize_byte_lenenc(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    let (length, index) = deserialize_int_3(&buf, &index);
    (Bytes::from(&buf[index..index + length as usize]), index + length as usize)
}

#[inline]
pub fn deserialize_byte_eof(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    (Bytes::from(&buf[*index..]), buf.len())
}
