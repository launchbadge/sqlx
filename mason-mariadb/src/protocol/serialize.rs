use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use bytes::Bytes;
use failure::Error;
use failure::err_msg;

const U24_MAX: usize = 0xFF_FF_FF;

#[inline]
pub fn serialize_length(buf: &mut Vec<u8>) {
    let mut length =  [0;  3];
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
pub fn serialize_int_8(buf: &mut Vec<u8>, value: u64) {
    buf.write_u64::<LittleEndian>(value).unwrap();
}

#[inline]
pub fn serialize_int_4(buf: &mut Vec<u8>, value: u32) {
    buf.write_u32::<LittleEndian>(value).unwrap();
}

#[inline]
pub fn serialize_int_3(buf: &mut Vec<u8>, value: u32) {
    buf.write_u24::<LittleEndian>(value).unwrap();
}

#[inline]
pub fn serialize_int_2(buf: &mut Vec<u8>, value: u16) {
    buf.write_u16::<LittleEndian>(value).unwrap();
}

#[inline]
pub fn serialize_int_1(buf: &mut Vec<u8>, value: u8) {
    buf.write_u8(value);
}

#[inline]
pub fn serialize_int_lenenc(buf: &mut Vec<u8>, value: Option<usize>) {
    if let Some(value) = value {
        if value > U24_MAX && value <= std::u64::MAX as usize{
            buf.write_u8(0xFE);
            serialize_int_8(buf, value as u64);
        } else if value > std::u16::MAX as usize && value <= U24_MAX {
            buf.write_u8(0xFD);
            serialize_int_3(buf, value as u32);
        } else if value > std::u8::MAX as usize && value <= std::u16::MAX as usize{
            buf.write_u8(0xFC);
            serialize_int_2(buf, value as u16);
        } else if value >= 0 && value <= std::u8::MAX as usize {
            buf.write_u8(0xFA);
            serialize_int_1(buf, value as u8);
        } else {
            panic!("Value is too long");
        }
    } else {
        buf.write_u8(0xFB);
    }
}

#[inline]
pub fn serialize_string_lenenc(buf: &mut Vec<u8>, string: &'static str) {
    if string.len() > 0xFFF {
        panic!("String inside string lenenc serialization is too long");
    }

    serialize_int_3(buf, string.len() as u32);
    if string.len() > 0 {
        buf.extend_from_slice(string.as_bytes());
    }
}

#[inline]
pub fn serialize_string_fix(buf: &mut Vec<u8>, string: &'static str, size: usize) {
    if size != string.len() {
        panic!("Sizes do not match");
    }
    buf.extend_from_slice(string.as_bytes());
}

#[inline]
pub fn serialize_string_null(buf: &mut Vec<u8>, string: &'static str) {
    buf.extend_from_slice(string.as_bytes());
    buf.write_u8(0);
}

#[inline]
pub fn serialize_string_eof(buf: &mut Vec<u8>, string: &'static str) {
    // Ignore the null terminator
    buf.extend_from_slice(string.as_bytes());
}

#[inline]
pub fn serialize_byte_lenenc(buf: &mut Vec<u8>, bytes: &Bytes) {
    if bytes.len() > 0xFFF {
        panic!("String inside string lenenc serialization is too long");
    }

    serialize_int_3(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

#[inline]
pub fn serialize_byte_fix(buf: &mut Vec<u8>, bytes: &Bytes, size: usize) {
    if size != bytes.len() {
        panic!("Sizes do not match");
    }

    buf.extend_from_slice(bytes);
}

#[inline]
pub fn serialize_byte_eof(buf: &mut Vec<u8>, bytes: &Bytes) {
    buf.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

    // [X] serialize_int_lenenc_u64
    // [X] serialize_int_lenenc_u32
    // [X] serialize_int_lenenc_u24
    // [X] serialize_int_lenenc_u16
    // [X] serialize_int_lenenc_u8
    // [X] serialize_int_u64
    // [X] serialize_int_u32
    // [X] serialize_int_u24
    // [X] serialize_int_u16
    // [X] serialize_int_u8
    // [X] serialize_string_lenenc
    // [X] serialize_string_fix
    // [X] serialize_string_null
    // [X] serialize_string_eof
    // [X] serialize_byte_lenenc
    // [X] serialize_byte_fix
    // [X] serialize_byte_eof

    #[test]
    fn it_encodes_length() {
        let mut buf: Vec<u8> = Vec::new();
        // Reserve space of length
        buf.write_u24::<LittleEndian>(0);
        // Sequence number; typically 0
        buf.write_u8(0x00);
        // Contents of buffer
        buf.write_u8(0xFF);
        serialize_length(&mut buf);

        assert_eq!(buf, b"\x01\0\0\0\xFF".to_vec());
    }

    #[test]
    fn it_encodes_int_lenenc_none() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_int_lenenc(&mut buf, None);

        assert_eq!(buf, b"\xFB".to_vec());
    }

    #[test]
    fn it_encodes_int_lenenc_u8() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_int_lenenc(&mut buf, Some(std::u8::MAX as usize));

        assert_eq!(buf, b"\xFA\xFF".to_vec());
    }

    #[test]
    fn it_encodes_int_lenenc_u16() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_int_lenenc(&mut buf, Some(std::u16::MAX as usize));

        assert_eq!(buf, b"\xFC\xFF\xFF".to_vec());
    }

    #[test]
    fn it_encodes_int_lenenc_u24() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_int_lenenc(&mut buf, Some(U24_MAX));

        assert_eq!(buf, b"\xFD\xFF\xFF\xFF".to_vec());
    }

    #[test]
    fn it_encodes_int_lenenc_u64() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_int_lenenc(&mut buf, Some(std::u64::MAX as usize));

        assert_eq!(buf, b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec());
    }

    #[test]
    fn it_encodes_int_u64() {
        let mut buf = Vec::new();
        serialize_int_8(&mut buf, std::u64::MAX);

        assert_eq!(buf, b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec());
    }


    #[test]
    fn it_encodes_int_u32() {
        let mut buf = Vec::new();
        serialize_int_4(&mut buf, std::u32::MAX);

        assert_eq!(buf, b"\xFF\xFF\xFF\xFF".to_vec());
    }


    #[test]
    fn it_encodes_int_u24() {
        let mut buf = Vec::new();
        serialize_int_3(&mut buf, U24_MAX as u32);

        assert_eq!(buf.to_vec(), b"\xFF\xFF\xFF".to_vec());
    }


    #[test]
    fn it_encodes_int_u16() {
        let mut buf = Vec::new();
        serialize_int_2(&mut buf, std::u16::MAX);

        assert_eq!(buf.to_vec(), b"\xFF\xFF".to_vec());
    }


    #[test]
    fn it_encodes_int_u8() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_int_1(&mut buf, std::u8::MAX);

        assert_eq!(buf, b"\xFF".to_vec());
    }

    #[test]
    fn it_encodes_string_lenenc() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_string_lenenc(&mut buf, "random_string");

        assert_eq!(buf, b"\x0D\x00\x00random_string".to_vec());
    }

    #[test]
    fn it_encodes_string_fix() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_string_fix(&mut buf, "random_string", 13);

        assert_eq!(buf, b"random_string".to_vec());
    }

    #[test]
    fn it_encodes_string_null() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_string_null(&mut buf, "random_string");

        assert_eq!(buf, b"random_string\0".to_vec());
    }


    #[test]
    fn it_encodes_string_eof() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_string_eof(&mut buf, "random_string");

        assert_eq!(buf, b"random_string".to_vec());
    }

    #[test]
    fn it_encodes_byte_lenenc() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_byte_lenenc(&mut buf, &Bytes::from("random_string"));

        assert_eq!(buf, b"\x0D\x00\x00random_string".to_vec());
    }

    #[test]
    fn it_encodes_byte_fix() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_byte_fix(&mut buf, &Bytes::from("random_string"), 13);

        assert_eq!(buf, b"random_string".to_vec());
    }

    #[test]
    fn it_encodes_byte_eof() {
        let mut buf: Vec<u8> = Vec::new();
        serialize_byte_eof(&mut buf, &Bytes::from("random_string"));

        assert_eq!(buf, b"random_string".to_vec());
    }
}
