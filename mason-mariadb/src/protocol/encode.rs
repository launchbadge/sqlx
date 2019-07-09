use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};

const U24_MAX: usize = 0xFF_FF_FF;

#[inline]
pub fn encode_length(buf: &mut BytesMut) {
    let mut length = [0; 3];
    if buf.len() > U24_MAX {
        panic!("Buffer too long");
    } else if buf.len() <= 4 {
        panic!("Buffer too short. Only contains packet length and sequence number")
    }

    LittleEndian::write_u24(&mut length, buf.len() as u32 - 4);

    // Set length at the start of the buffer
    // sadly there is no `prepend` for rust Vec
    buf[0] = length[0];
    buf[1] = length[1];
    buf[2] = length[2];
}

#[inline]
pub fn encode_int_8(buf: &mut BytesMut, value: u64) {
    buf.put_u64_le(value);
}

#[inline]
pub fn encode_int_4(buf: &mut BytesMut, value: u32) {
    buf.put_u32_le(value);
}

#[inline]
pub fn encode_int_3(buf: &mut BytesMut, value: u32) {
    let length = value.to_le_bytes();
    buf.extend_from_slice(&length[0..3]);
}

#[inline]
pub fn encode_int_2(buf: &mut BytesMut, value: u16) {
    buf.put_u16_le(value);
}

#[inline]
pub fn encode_int_1(buf: &mut BytesMut, value: u8) {
    buf.put_u8(value);
}

#[inline]
pub fn encode_int_lenenc(buf: &mut BytesMut, value: Option<&usize>) {
    if let Some(value) = value {
        if *value > U24_MAX && *value <= std::u64::MAX as usize {
            buf.put_u8(0xFE);
            encode_int_8(buf, *value as u64);
        } else if *value > std::u16::MAX as usize && *value <= U24_MAX {
            buf.put_u8(0xFD);
            encode_int_3(buf, *value as u32);
        } else if *value > std::u8::MAX as usize && *value <= std::u16::MAX as usize {
            buf.put_u8(0xFC);
            encode_int_2(buf, *value as u16);
        } else if *value <= std::u8::MAX as usize {
            buf.put_u8(0xFA);
            encode_int_1(buf, *value as u8);
        } else {
            panic!("Value is too long");
        }
    } else {
        buf.put_u8(0xFB);
    }
}

#[inline]
pub fn encode_string_lenenc(buf: &mut BytesMut, string: &Bytes) {
    if string.len() > 0xFFF {
        panic!("String inside string lenenc serialization is too long");
    }

    encode_int_3(buf, string.len() as u32);
    if string.len() > 0 {
        buf.extend_from_slice(string);
    }
}

#[inline]
pub fn encode_string_null(buf: &mut BytesMut, string: &Bytes) {
    buf.extend_from_slice(string);
    buf.put(0_u8);
}

#[inline]
pub fn encode_string_fix(buf: &mut BytesMut, bytes: &Bytes, size: usize) {
    if size != bytes.len() {
        panic!("Sizes do not match");
    }

    buf.extend_from_slice(bytes);
}

#[inline]
pub fn encode_string_eof(buf: &mut BytesMut, bytes: &Bytes) {
    buf.extend_from_slice(bytes);
}

#[inline]
pub fn encode_byte_lenenc(buf: &mut BytesMut, bytes: &Bytes) {
    if bytes.len() > 0xFFF {
        panic!("String inside string lenenc serialization is too long");
    }

    encode_int_3(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

#[inline]
pub fn encode_byte_fix(buf: &mut BytesMut, bytes: &Bytes, size: usize) {
    if size != bytes.len() {
        panic!("Sizes do not match");
    }

    buf.extend_from_slice(bytes);
}

#[inline]
pub fn encode_byte_eof(buf: &mut BytesMut, bytes: &Bytes) {
    buf.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    // [X] encode_int_lenenc_u64
    // [X] encode_int_lenenc_u32
    // [X] encode_int_lenenc_u24
    // [X] encode_int_lenenc_u16
    // [X] encode_int_lenenc_u8
    // [X] encode_int_u64
    // [X] encode_int_u32
    // [X] encode_int_u24
    // [X] encode_int_u16
    // [X] encode_int_u8
    // [X] encode_string_lenenc
    // [X] encode_string_fix
    // [X] encode_string_null
    // [X] encode_string_eof
    // [X] encode_byte_lenenc
    // [X] encode_byte_fix
    // [X] encode_byte_eof

    #[test]
    fn it_encodes_int_lenenc_none() {
        let mut buf = BytesMut::new();
        encode_int_lenenc(&mut buf, None);

        assert_eq!(&buf[..], b"\xFB");
    }

    #[test]
    fn it_encodes_int_lenenc_u8() {
        let mut buf = BytesMut::new();
        encode_int_lenenc(&mut buf, Some(&(std::u8::MAX as usize)));

        assert_eq!(&buf[..], b"\xFA\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u16() {
        let mut buf = BytesMut::new();
        encode_int_lenenc(&mut buf, Some(&(std::u16::MAX as usize)));

        assert_eq!(&buf[..], b"\xFC\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u24() {
        let mut buf = BytesMut::new();
        encode_int_lenenc(&mut buf, Some(&U24_MAX));

        assert_eq!(&buf[..], b"\xFD\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u64() {
        let mut buf = BytesMut::new();
        encode_int_lenenc(&mut buf, Some(&(std::u64::MAX as usize)));

        assert_eq!(&buf[..], b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u64() {
        let mut buf = BytesMut::new();
        encode_int_8(&mut buf, std::u64::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u32() {
        let mut buf = BytesMut::new();
        encode_int_4(&mut buf, std::u32::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u24() {
        let mut buf = BytesMut::new();
        encode_int_3(&mut buf, U24_MAX as u32);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u16() {
        let mut buf = BytesMut::new();
        encode_int_2(&mut buf, std::u16::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u8() {
        let mut buf = BytesMut::new();
        encode_int_1(&mut buf, std::u8::MAX);

        assert_eq!(&buf[..], b"\xFF");
    }

    #[test]
    fn it_encodes_string_lenenc() {
        let mut buf = BytesMut::new();
        encode_string_lenenc(&mut buf, &Bytes::from_static(b"random_string"));

        assert_eq!(&buf[..], b"\x0D\x00\x00random_string");
    }

    #[test]
    fn it_encodes_string_fix() {
        let mut buf = BytesMut::new();
        encode_string_fix(&mut buf, &Bytes::from_static(b"random_string"), 13);

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_string_null() {
        let mut buf = BytesMut::new();
        encode_string_null(&mut buf, &Bytes::from_static(b"random_string"));

        assert_eq!(&buf[..], b"random_string\0");
    }

    #[test]
    fn it_encodes_string_eof() {
        let mut buf = BytesMut::new();
        encode_string_eof(&mut buf, &Bytes::from_static(b"random_string"));

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_byte_lenenc() {
        let mut buf = BytesMut::new();
        encode_byte_lenenc(&mut buf, &Bytes::from("random_string"));

        assert_eq!(&buf[..], b"\x0D\x00\x00random_string");
    }

    #[test]
    fn it_encodes_byte_fix() {
        let mut buf = BytesMut::new();
        encode_byte_fix(&mut buf, &Bytes::from("random_string"), 13);

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_byte_eof() {
        let mut buf = BytesMut::new();
        encode_byte_eof(&mut buf, &Bytes::from("random_string"));

        assert_eq!(&buf[..], b"random_string");
    }
}
