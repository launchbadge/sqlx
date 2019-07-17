use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};

const U24_MAX: usize = 0xFF_FF_FF;

pub struct Encoder {
    pub buf: BytesMut,
}

impl Encoder {
    pub fn new(capacity: usize) -> Self {
        Encoder { buf: BytesMut::with_capacity(capacity) }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    // Reserve space for packet header; Packet Body Length (3 bytes) and sequence number (1 byte)
    #[inline]
    pub fn alloc_packet_header(&mut self) {
        self.buf.extend_from_slice(&[0; 4]);
    }

    #[inline]
    pub fn seq_no(&mut self, seq_no: u8) {
        self.buf[3] = seq_no;
    }

    #[inline]
    pub fn encode_length(&mut self) {
        let mut length = [0; 3];
        if self.buf.len() > U24_MAX {
            panic!("Buffer too long");
        } else if self.buf.len() <= 4 {
            panic!("Buffer too short. Only contains packet length and sequence number")
        }

        LittleEndian::write_u24(&mut length, self.buf.len() as u32 - 4);

        // Set length at the start of the buffer
        // sadly there is no `prepend` for rust Vec
        self.buf[0] = length[0];
        self.buf[1] = length[1];
        self.buf[2] = length[2];
    }

    #[inline]
    pub fn encode_int_8(&mut self, value: u64) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    pub fn encode_int_4(&mut self, value: u32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    pub fn encode_int_3(&mut self, value: u32)  {
        self.buf.extend_from_slice(&value.to_le_bytes()[0..3]);
    }

    #[inline]
    pub fn encode_int_2(&mut self, value: u16) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    pub fn encode_int_1(&mut self, value: u8) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    #[inline]
    pub fn encode_int_lenenc(&mut self, value: Option<&usize>) {
        if let Some(value) = value {
            if *value > U24_MAX && *value <= std::u64::MAX as usize {
                self.buf.push(0xFE);
                self.encode_int_8(*value as u64);
            } else if *value > std::u16::MAX as usize && *value <= U24_MAX {
                self.buf.push(0xFD);
                self.encode_int_3(*value as u32);
            } else if *value > std::u8::MAX as usize && *value <= std::u16::MAX as usize {
                self.buf.push(0xFC);
                self.encode_int_2(*value as u16);
            } else if *value <= std::u8::MAX as usize {
                self.buf.push(0xFA);
                self.encode_int_1(*value as u8);
            } else {
                panic!("Value is too long");
            }
        } else {
            self.buf.push(0xFB);
        }
    }

    #[inline]
    pub fn encode_string_lenenc(&mut self, string: &Bytes) {
        if string.len() > 0xFFF {
            panic!("String inside string lenenc serialization is too long");
        }

        self.encode_int_3(string.len() as u32);
        if string.len() > 0 {
            self.buf.extend_from_slice(string);
        }
    }

    #[inline]
    pub fn encode_string_null(&mut self, string: &Bytes) {
        self.buf.extend_from_slice(string);
        self.buf.put(0_u8);
    }

    #[inline]
    pub fn encode_string_fix(&mut self, bytes: &Bytes, size: usize) {
        if size != bytes.len() {
            panic!("Sizes do not match");
        }

        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_string_eof(&mut self, bytes: &Bytes) {
        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_byte_lenenc(&mut self, bytes: &Bytes) {
        if bytes.len() > 0xFFF {
            panic!("String inside string lenenc serialization is too long");
        }

        self.encode_int_3(bytes.len() as u32);
        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_byte_fix(&mut self, bytes: &Bytes, size: usize) {
        if size != bytes.len() {
            panic!("Sizes do not match");
        }

        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_byte_eof(&mut self, bytes: &Bytes) {
        self.buf.extend_from_slice(bytes);
    }
}

impl From<BytesMut> for Encoder {
    fn from(buf: BytesMut) -> Encoder {
        Encoder { buf }
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
        let mut encoder = Encoder::new(128);
        encoder.encode_int_lenenc(None);

        assert_eq!(&encoder.buf[..], b"\xFB");
    }

    #[test]
    fn it_encodes_int_lenenc_u8() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_lenenc(Some(&(std::u8::MAX as usize)));

        assert_eq!(&encoder.buf[..], b"\xFA\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u16() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_lenenc(Some(&(std::u16::MAX as usize)));

        assert_eq!(&encoder.buf[..], b"\xFC\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u24() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_lenenc(Some(&U24_MAX));

        assert_eq!(&encoder.buf[..], b"\xFD\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_lenenc_u64() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_lenenc(Some(&(std::u64::MAX as usize)));

        assert_eq!(&encoder.buf[..], b"\xFE\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u64() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_8(std::u64::MAX);

        assert_eq!(&encoder.buf[..], b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u32() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_4(std::u32::MAX);

        assert_eq!(&encoder.buf[..], b"\xFF\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u24() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_3(U24_MAX as u32);

        assert_eq!(&encoder.buf[..], b"\xFF\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u16() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_2(std::u16::MAX);

        assert_eq!(&encoder.buf[..], b"\xFF\xFF");
    }

    #[test]
    fn it_encodes_int_u8() {
        let mut encoder = Encoder::new(128);
        encoder.encode_int_1(std::u8::MAX);

        assert_eq!(&encoder.buf[..], b"\xFF");
    }

    #[test]
    fn it_encodes_string_lenenc() {
        let mut encoder = Encoder::new(128);
        encoder.encode_string_lenenc(&Bytes::from_static(b"random_string"));

        assert_eq!(&encoder.buf[..], b"\x0D\x00\x00random_string");
    }

    #[test]
    fn it_encodes_string_fix() {
        let mut encoder = Encoder::new(128);
        encoder.encode_string_fix(&Bytes::from_static(b"random_string"), 13);

        assert_eq!(&encoder.buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_string_null() {
        let mut encoder = Encoder::new(128);
        encoder.encode_string_null(&Bytes::from_static(b"random_string"));

        assert_eq!(&encoder.buf[..], b"random_string\0");
    }

    #[test]
    fn it_encodes_string_eof() {
        let mut encoder = Encoder::new(128);
        encoder.encode_string_eof(&Bytes::from_static(b"random_string"));

        assert_eq!(&encoder.buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_byte_lenenc() {
        let mut encoder = Encoder::new(128);
        encoder.encode_byte_lenenc(&Bytes::from("random_string"));

        assert_eq!(&encoder.buf[..], b"\x0D\x00\x00random_string");
    }

    #[test]
    fn it_encodes_byte_fix() {
        let mut encoder = Encoder::new(128);
        encoder.encode_byte_fix(&Bytes::from("random_string"), 13);

        assert_eq!(&encoder.buf[..], b"random_string");
    }

    #[test]
    fn it_encodes_byte_eof() {
        let mut encoder = Encoder::new(128);
        encoder.encode_byte_eof(&Bytes::from("random_string"));

        assert_eq!(&encoder.buf[..], b"random_string");
    }
}
