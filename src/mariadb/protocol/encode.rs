use crate::mariadb::{ConnContext, Connection, FieldType};
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use failure::Error;

pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error>;
}

pub const U24_MAX: usize = 0xFF_FF_FF;

pub trait BufMut {
    // Reserve space for packet header; Packet Body Length (3 bytes) and sequence number (1 byte)
    #[inline]
    fn alloc_packet_header(&mut self);

    // Encode the sequence number; the 4th byte of the packet
    #[inline]
    fn seq_no(&mut self, seq_no: u8);

    // Encode the sequence number; the first 3 bytes of the packet in little endian format
    #[inline]
    fn put_length(&mut self);

    // Encode a u64 as an int<8>
    #[inline]
    fn put_int_u64(&mut self, value: u64);

    // Encode a i64 as an int<8>
    #[inline]
    fn put_int_i64(&mut self, value: i64);

    #[inline]
    fn put_int_8(&mut self, bytes: &Bytes);

    // Encode a u32 as an int<4>
    #[inline]
    fn put_int_u32(&mut self, value: u32);

    // Encode a i32 as an int<4>
    #[inline]
    fn put_int_i32(&mut self, value: i32);

    #[inline]
    fn put_int_4(&mut self, bytes: &Bytes);

    // Encode a u32 (truncated to u24) as an int<3>
    #[inline]
    fn put_int_u24(&mut self, value: u32);

    // Encode a i32 (truncated to i24) as an int<3>
    #[inline]
    fn put_int_i24(&mut self, value: i32);

    // Encode a u16 as an int<2>
    #[inline]
    fn put_int_u16(&mut self, value: u16);

    // Encode a i16 as an int<2>
    #[inline]
    fn put_int_i16(&mut self, value: i16);

    #[inline]
    fn put_int_2(&mut self, bytes: &Bytes);

    // Encode a u8 as an int<1>
    #[inline]
    fn put_int_u8(&mut self, value: u8);

    #[inline]
    fn put_int_1(&mut self, bytes: &Bytes);

    // Encode a i8 as an int<1>
    #[inline]
    fn put_int_i8(&mut self, value: i8);

    // Encode an int<lenenc>; length putd int
    // See Decoder::decode_int_lenenc for explanation of how int<lenenc> is putd
    #[inline]
    fn put_int_lenenc(&mut self, value: Option<&usize>);

    // Encode a string<lenenc>; a length putd string.
    #[inline]
    fn put_string_lenenc(&mut self, string: &Bytes);

    // Encode a string<null>; a null termianted string (C style)
    #[inline]
    fn put_string_null(&mut self, string: &Bytes);

    // TODO: Combine with previous method
    fn put_str_null(&mut self, string: &str);

    // Encode a string<fix>; a string of fixed length
    #[inline]
    fn put_string_fix(&mut self, bytes: &Bytes, size: usize);

    // Encode a string<eof>; a string that is terminated by the packet length
    #[inline]
    fn put_string_eof(&mut self, bytes: &Bytes);

    // Same as the string counterpart copied to maintain consistency with the spec.
    #[inline]
    fn put_byte_lenenc(&mut self, bytes: &Bytes);

    // Same as the string counterpart copied to maintain consistency with the spec.
    #[inline]
    fn put_byte_fix(&mut self, bytes: &Bytes, size: usize);

    // Same as the string counterpart copied to maintain consistency with the spec.
    #[inline]
    fn put_byte_eof(&mut self, bytes: &Bytes);

    #[inline]
    fn put_param(&mut self, bytes: &Bytes, ty: &FieldType);
}

impl BufMut for Vec<u8> {
    // Reserve space for packet header; Packet Body Length (3 bytes) and sequence number (1 byte)
    #[inline]
    fn alloc_packet_header(&mut self) {
        self.extend_from_slice(&[0; 4]);
    }

    // Encode the sequence number; the 4th byte of the packet
    #[inline]
    fn seq_no(&mut self, seq_no: u8) {
        self[3] = seq_no;
    }

    // Encode the sequence number; the first 3 bytes of the packet in little endian format
    #[inline]
    fn put_length(&mut self) {
        let mut length = [0; 3];
        if self.len() > U24_MAX {
            panic!("Buffer too long");
        } else if self.len() < 4 {
            panic!("Buffer too short. Only contains packet length and sequence number")
        }

        LittleEndian::write_u24(&mut length, self.len() as u32 - 4);

        // Set length at the start of thefer
        // sadly there is no `prepend` for rust Vec
        self[0] = length[0];
        self[1] = length[1];
        self[2] = length[2];
    }

    // Encode a u64 as an int<8>
    #[inline]
    fn put_int_u64(&mut self, value: u64) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    // Encode a i64 as an int<8>
    #[inline]
    fn put_int_i64(&mut self, value: i64) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_int_8(&mut self, bytes: &Bytes) {
        self.extend_from_slice(bytes);
    }

    // Encode a u32 as an int<4>
    #[inline]
    fn put_int_u32(&mut self, value: u32) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    // Encode a i32 as an int<4>
    #[inline]
    fn put_int_i32(&mut self, value: i32) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_int_4(&mut self, bytes: &Bytes) {
        self.extend_from_slice(bytes);
    }

    // Encode a u32 (truncated to u24) as an int<3>
    #[inline]
    fn put_int_u24(&mut self, value: u32) {
        self.extend_from_slice(&value.to_le_bytes()[0..3]);
    }

    // Encode a i32 (truncated to i24) as an int<3>
    #[inline]
    fn put_int_i24(&mut self, value: i32) {
        self.extend_from_slice(&value.to_le_bytes()[0..3]);
    }

    // Encode a u16 as an int<2>
    #[inline]
    fn put_int_u16(&mut self, value: u16) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    // Encode a i16 as an int<2>
    #[inline]
    fn put_int_i16(&mut self, value: i16) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_int_2(&mut self, bytes: &Bytes) {
        self.extend_from_slice(bytes);
    }

    // Encode a u8 as an int<1>
    #[inline]
    fn put_int_u8(&mut self, value: u8) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn put_int_1(&mut self, bytes: &Bytes) {
        self.extend_from_slice(bytes);
    }

    // Encode a i8 as an int<1>
    #[inline]
    fn put_int_i8(&mut self, value: i8) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    // Encode an int<lenenc>; length putd int
    // See Decoder::decode_int_lenenc for explanation of how int<lenenc> is putd
    #[inline]
    fn put_int_lenenc(&mut self, value: Option<&usize>) {
        if let Some(value) = value {
            if *value > U24_MAX && *value <= std::u64::MAX as usize {
                self.push(0xFE);
                self.put_int_u64(*value as u64);
            } else if *value > std::u16::MAX as usize && *value <= U24_MAX {
                self.push(0xFD);
                self.put_int_u24(*value as u32);
            } else if *value > std::u8::MAX as usize && *value <= std::u16::MAX as usize {
                self.push(0xFC);
                self.put_int_u16(*value as u16);
            } else if *value <= std::u8::MAX as usize {
                match *value {
                    // If the value is of size u8 and one of the key bytes used in length encoding
                    // we must put that single byte as a u16
                    0xFB | 0xFC | 0xFD | 0xFE | 0xFF => {
                        self.push(0xFC);
                        self.push(*value as u8);
                        self.push(0);
                    }

                    v => self.push(v as u8),
                }
            } else {
                panic!("Value is too long");
            }
        } else {
            self.push(0xFB);
        }
    }

    // Encode a string<lenenc>; a length putd string.
    #[inline]
    fn put_string_lenenc(&mut self, string: &Bytes) {
        if string.len() > 0xFFF {
            panic!("String inside string lenenc serialization is too long");
        }

        self.put_int_lenenc(Some(&string.len()));
        if string.len() > 0 {
            self.extend_from_slice(string);
        }
    }

    // Encode a string<null>; a null terminated string (C style)
    #[inline]
    fn put_string_null(&mut self, string: &Bytes) {
        self.extend_from_slice(string);
        self.push(0_u8);
    }

    // TODO: Combine this method with the previous
    // Encode a string<null>; a null terminated string (C style)
    #[inline]
    fn put_str_null(&mut self, string: &str) {
        self.extend_from_slice(string.as_bytes());
        self.push(0_u8);
    }

    // Encode a string<fix>; a string of fixed length
    #[inline]
    fn put_string_fix(&mut self, bytes: &Bytes, size: usize) {
        if size != bytes.len() {
            panic!("Sizes do not match");
        }

        self.extend_from_slice(bytes);
    }

    // Encode a string<eof>; a string that is terminated by the packet length
    #[inline]
    fn put_string_eof(&mut self, bytes: &Bytes) {
        self.extend_from_slice(bytes);
    }

    // Same as the string counterpart copied to maintain consistency with the spec.
    #[inline]
    fn put_byte_lenenc(&mut self, bytes: &Bytes) {
        self.put_int_lenenc(Some(&bytes.len()));
        if bytes.len() > 0 {
            self.extend_from_slice(bytes);
        }
    }

    // Same as the string counterpart copied to maintain consistency with the spec.
    #[inline]
    fn put_byte_fix(&mut self, bytes: &Bytes, size: usize) {
        assert_eq!(size, bytes.len());

        self.extend_from_slice(bytes);
    }

    // Same as the string counterpart copied to maintain consistency with the spec.
    #[inline]
    fn put_byte_eof(&mut self, bytes: &Bytes) {
        self.extend_from_slice(bytes);
    }

    #[inline]
    fn put_param(&mut self, bytes: &Bytes, ty: &FieldType) {
        match ty {
            FieldType::MysqlTypeDecimal => self.put_string_lenenc(bytes),
            FieldType::MysqlTypeTiny => self.put_int_1(bytes),
            FieldType::MysqlTypeShort => self.put_int_2(bytes),
            FieldType::MysqlTypeLong => self.put_int_4(bytes),
            FieldType::MysqlTypeFloat => self.put_int_4(bytes),
            FieldType::MysqlTypeDouble => self.put_int_8(bytes),
            FieldType::MysqlTypeNull => panic!("Type cannot be FieldType::MysqlTypeNull"),
            FieldType::MysqlTypeTimestamp => unimplemented!(),
            FieldType::MysqlTypeLonglong => self.put_int_8(bytes),
            FieldType::MysqlTypeInt24 => self.put_int_4(bytes),
            FieldType::MysqlTypeDate => unimplemented!(),
            FieldType::MysqlTypeTime => unimplemented!(),
            FieldType::MysqlTypeDatetime => unimplemented!(),
            FieldType::MysqlTypeYear => self.put_int_4(bytes),
            FieldType::MysqlTypeNewdate => unimplemented!(),
            FieldType::MysqlTypeVarchar => self.put_string_lenenc(bytes),
            FieldType::MysqlTypeBit => self.put_string_lenenc(bytes),
            FieldType::MysqlTypeTimestamp2 => unimplemented!(),
            FieldType::MysqlTypeDatetime2 => unimplemented!(),
            FieldType::MysqlTypeTime2 => unimplemented!(),
            FieldType::MysqlTypeJson => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeNewdecimal => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeEnum => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeSet => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeTinyBlob => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeMediumBlob => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeLongBlob => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeBlob => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeVarString => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeString => self.put_byte_lenenc(bytes),
            FieldType::MysqlTypeGeometry => self.put_byte_lenenc(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [X] it_encodes_int_lenenc_u64
    // [X] it_encodes_int_lenenc_u32
    // [X] it_encodes_int_lenenc_u24
    // [X] it_encodes_int_lenenc_u16
    // [X] it_encodes_int_lenenc_u8
    // [X] it_encodes_int_u64
    // [X] it_encodes_int_u32
    // [X] it_encodes_int_u24
    // [X] it_encodes_int_u16
    // [X] it_encodes_int_u8
    // [X] it_encodes_string_lenenc
    // [X] it_encodes_string_fix
    // [X] it_encodes_string_null
    // [X] it_encodes_string_eof
    // [X] it_encodes_byte_lenenc
    // [X] it_encodes_byte_fix
    // [X] it_encodes_byte_eof

    #[test]
    fn it_encodes_int_lenenc_none() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(None);

        assert_eq!(&buf[..], b"\xFB");
    }

    #[test]
    fn it_encodes_int_lenenc_u8() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(0xFA as usize)));

        assert_eq!(&buf[..], b"\xFA");
    }

    #[test]
    fn it_encodes_int_lenenc_u16() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(std::u16::MAX as usize)));

        assert_eq!(&buf[..], b"\xFC\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u24() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&U24_MAX));

        assert_eq!(&buf[..], b"\xFD\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u64() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(std::u64::MAX as usize)));

        assert_eq!(&buf[..], b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_fb() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(0xFB as usize)));

        assert_eq!(&buf[..], b"\xFC\xFB\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(0xFC as usize)));

        assert_eq!(&buf[..], b"\xFC\xFC\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fd() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(0xFD as usize)));

        assert_eq!(&buf[..], b"\xFC\xFD\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fe() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(0xFE as usize)));

        assert_eq!(&buf[..], b"\xFC\xFE\x00");
    }

    fn it_encodes_int_lenenc_ff() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_lenenc(Some(&(0xFF as usize)));

        assert_eq!(&buf[..], b"\xFC\xFF\x00");
    }

    #[test]
    fn it_encodes_int_u64() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_u64(std::u64::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u32() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_u32(std::u32::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u24() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_u24(U24_MAX as u32);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u16() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_u16(std::u16::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u8() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_int_u8(std::u8::MAX);

        assert_eq!(&buf[..], b"\xFF");
    }

    #[test]
    fn it_encodes_string_lenenc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_string_lenenc(&Bytes::from_static(b"random_string"));

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }

    #[test]
    fn it_encodes_string_fix() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_string_fix(&Bytes::from_static(b"random_string"), 13);

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_string_null() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_string_null(&Bytes::from_static(b"random_string"));

        assert_eq!(&buf[..], b"random_string\0");
    }

    #[test]
    fn it_encodes_string_eof() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_string_eof(&Bytes::from_static(b"random_string"));

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_byte_lenenc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_byte_lenenc(&Bytes::from("random_string"));

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }

    #[test]
    fn it_encodes_byte_fix() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_byte_fix(&Bytes::from("random_string"), 13);

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_byte_eof() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_byte_eof(&Bytes::from("random_string"));

        assert_eq!(&buf[..], b"random_string");
    }
}
