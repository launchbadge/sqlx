pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>);
}

pub const U24_MAX: usize = 0xFF_FF_FF;

//    #[inline]
//    fn put_param(&mut self, bytes: &Bytes, ty: FieldType) {
//        match ty {
//            FieldType::MYSQL_TYPE_DECIMAL => self.put_string_lenenc(bytes),
//            FieldType::MYSQL_TYPE_TINY => self.put_int_1(bytes),
//            FieldType::MYSQL_TYPE_SHORT => self.put_int_2(bytes),
//            FieldType::MYSQL_TYPE_LONG => self.put_int_4(bytes),
//            FieldType::MYSQL_TYPE_FLOAT => self.put_int_4(bytes),
//            FieldType::MYSQL_TYPE_DOUBLE => self.put_int_8(bytes),
//            FieldType::MYSQL_TYPE_NULL => panic!("Type cannot be FieldType::MysqlTypeNull"),
//            FieldType::MYSQL_TYPE_TIMESTAMP => unimplemented!(),
//            FieldType::MYSQL_TYPE_LONGLONG => self.put_int_8(bytes),
//            FieldType::MYSQL_TYPE_INT24 => self.put_int_4(bytes),
//            FieldType::MYSQL_TYPE_DATE => unimplemented!(),
//            FieldType::MYSQL_TYPE_TIME => unimplemented!(),
//            FieldType::MYSQL_TYPE_DATETIME => unimplemented!(),
//            FieldType::MYSQL_TYPE_YEAR => self.put_int_4(bytes),
//            FieldType::MYSQL_TYPE_NEWDATE => unimplemented!(),
//            FieldType::MYSQL_TYPE_VARCHAR => self.put_string_lenenc(bytes),
//            FieldType::MYSQL_TYPE_BIT => self.put_string_lenenc(bytes),
//            FieldType::MYSQL_TYPE_TIMESTAMP2 => unimplemented!(),
//            FieldType::MYSQL_TYPE_DATETIME2 => unimplemented!(),
//            FieldType::MYSQL_TYPE_TIME2 => unimplemented!(),
//            FieldType::MYSQL_TYPE_JSON => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_NEWDECIMAL => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_ENUM => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_SET => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_TINY_BLOB => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_MEDIUM_BLOB => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_LONG_BLOB => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_BLOB => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_VAR_STRING => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_STRING => self.put_byte_lenenc(bytes),
//            FieldType::MYSQL_TYPE_GEOMETRY => self.put_byte_lenenc(bytes),
//            _ => panic!("Unrecognized field type"),
//        }
//    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::BufMutExt;
    use crate::io::BufMut;

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
        buf.put_u64_lenenc(Some(0u64));

        assert_eq!(&buf[..], b"\xFB");
    }

    #[test]
    fn it_encodes_int_lenenc_u8() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(0xFA as u64));

        assert_eq!(&buf[..], b"\xFA");
    }

    #[test]
    fn it_encodes_int_lenenc_u16() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(std::u16::MAX as u64));

        assert_eq!(&buf[..], b"\xFC\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u24() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(U24_MAX as u64));

        assert_eq!(&buf[..], b"\xFD\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u64() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(std::u64::MAX));

        assert_eq!(&buf[..], b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_fb() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(0xFB as u64));

        assert_eq!(&buf[..], b"\xFC\xFB\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(0xFC as u64));

        assert_eq!(&buf[..], b"\xFC\xFC\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fd() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(0xFD as u64));

        assert_eq!(&buf[..], b"\xFC\xFD\x00");
    }

    #[test]
    fn it_encodes_int_lenenc_fe() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(0xFE as u64));

        assert_eq!(&buf[..], b"\xFC\xFE\x00");
    }

    fn it_encodes_int_lenenc_ff() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64_lenenc(Some(0xFF as u64));

        assert_eq!(&buf[..], b"\xFC\xFF\x00");
    }

    #[test]
    fn it_encodes_int_u64() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u64(std::u64::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u32() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u32(std::u32::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u24() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u24(U24_MAX as u32);

        assert_eq!(&buf[..], b"\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u16() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u16(std::u16::MAX);

        assert_eq!(&buf[..], b"\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u8() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_u8(std::u8::MAX);

        assert_eq!(&buf[..], b"\xFF");
    }

    #[test]
    fn it_encodes_string_lenenc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_str_lenenc("random_string");

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }

    #[test]
    fn it_encodes_string_fix() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_str("random_string");

        assert_eq!(&buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_string_null() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_str_null("random_string");

        assert_eq!(&buf[..], b"random_string\0");
    }

    #[test]
    fn it_encodes_byte_lenenc() {
        let mut buf = Vec::with_capacity(1024);
        buf.put_byte_lenenc(b"random_string");

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }
}
