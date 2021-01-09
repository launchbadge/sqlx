pub(crate) trait MySqlWriteExt: sqlx_core::io::WriteExt {
    fn write_uint_lenenc(&mut self, value: u64);

    fn write_str_lenenc(&mut self, value: &str);

    fn write_bytes_lenenc(&mut self, value: &[u8]);
}

impl MySqlWriteExt for Vec<u8> {
    fn write_uint_lenenc(&mut self, value: u64) {
        // https://dev.mysql.com/doc/internals/en/integer.html
        // https://mariadb.com/kb/en/library/protocol-data-types/#length-encoded-integers

        if value < 251 {
            // if the value is < 251, it is stored as a 1-byte integer

            #[allow(clippy::cast_possible_truncation)]
            self.push(value as u8);
        } else if value < 0x1_00_00 {
            // if the value is ≥ 251 and < (2 ** 16), it is stored as fc + 2-byte integer
            self.reserve(3);
            self.push(0xfc);

            #[allow(clippy::cast_possible_truncation)]
            self.extend_from_slice(&(value as u16).to_le_bytes());
        } else if value < 0x1_00_00_00 {
            // if the value is ≥ (2 ** 16) and < (2 ** 24), it is stored as fd + 3-byte integer
            self.reserve(4);
            self.push(0xfd);

            #[allow(clippy::cast_possible_truncation)]
            self.extend_from_slice(&(value as u32).to_le_bytes()[..3]);
        } else {
            // if the value is ≥ (2 ** 24) and < (2 ** 64) it is stored as fe + 8-byte integer
            self.reserve(9);
            self.push(0xfe);
            self.extend_from_slice(&value.to_le_bytes());
        }
    }

    #[inline]
    fn write_str_lenenc(&mut self, value: &str) {
        self.write_bytes_lenenc(value.as_bytes());
    }

    fn write_bytes_lenenc(&mut self, value: &[u8]) {
        self.write_uint_lenenc(value.len() as u64);
        self.extend_from_slice(value);
    }
}

#[cfg(test)]
mod tests {
    use super::MySqlWriteExt;

    #[test]
    fn write_int_lenenc_u8() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFA as u64);

        assert_eq!(&buf[..], b"\xFA");
    }

    #[test]
    fn write_int_lenenc_u16() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(std::u16::MAX as u64);

        assert_eq!(&buf[..], b"\xFC\xFF\xFF");
    }

    #[test]
    fn write_int_lenenc_u24() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFF_FF_FF as u64);

        assert_eq!(&buf[..], b"\xFD\xFF\xFF\xFF");
    }

    #[test]
    fn write_int_lenenc_u64() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(std::u64::MAX);

        assert_eq!(&buf[..], b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn write_int_lenenc_fb() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFB as u64);

        assert_eq!(&buf[..], b"\xFC\xFB\x00");
    }

    #[test]
    fn write_int_lenenc_fc() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFC as u64);

        assert_eq!(&buf[..], b"\xFC\xFC\x00");
    }

    #[test]
    fn write_int_lenenc_fd() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFD as u64);

        assert_eq!(&buf[..], b"\xFC\xFD\x00");
    }

    #[test]
    fn write_int_lenenc_fe() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFE as u64);

        assert_eq!(&buf[..], b"\xFC\xFE\x00");
    }

    #[test]
    fn write_int_lenenc_ff() {
        let mut buf = Vec::new();
        buf.write_uint_lenenc(0xFF as u64);

        assert_eq!(&buf[..], b"\xFC\xFF\x00");
    }

    #[test]
    fn write_string_lenenc() {
        let mut buf = Vec::new();
        buf.write_str_lenenc("random_string");

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }

    #[test]
    fn write_byte_lenenc() {
        let mut buf = Vec::new();
        buf.write_bytes_lenenc(b"random_string");

        assert_eq!(&buf[..], b"\x0Drandom_string");
    }
}
