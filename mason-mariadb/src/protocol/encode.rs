use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use super::serialize::Serialize;
use super::types::Capabilities;
use failure::Error;

const U24_MAX: usize = 0xFF_FF_FF;

pub enum Encoder<'a> {
    Ref {
        buf: &'a mut BytesMut,
    },
    Owned {
        buf: BytesMut,
    }
}

impl<'a> Encoder<'a> {
    pub fn new(capacity: usize) -> Self {
        Encoder::Owned {
            buf: BytesMut::with_capacity(capacity)
        }
    }

    pub fn clear(&mut self) {
        match self {
            Encoder::Ref { buf } => buf.clear(),
            Encoder::Owned { buf } => buf.clear(),
        }
    }

    pub fn buf(&self) -> &BytesMut {
        match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        }
    }

    // Reserve space for packet header; Packet Body Length (3 bytes) and sequence number (1 byte)
    #[inline]
    pub fn alloc_packet_header(&mut self) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.extend_from_slice(&[0; 4]);
    }

    #[inline]
    pub fn seq_no(&mut self, seq_no: u8) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf[3] = seq_no;
    }

    #[inline]
    pub fn serialize<S: Serialize>(&mut self, message: S, capabilities: &Capabilities) -> Result<(), Error> {
        message.serialize(self, capabilities)

    }

    #[inline]
    pub fn encode_length(&mut self) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
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
    pub fn encode_int_8(&mut self, value: u64) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.put_u64_le(value);
    }

    #[inline]
    pub fn encode_int_4(&mut self, value: u32) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.put_u32_le(value);
    }

    #[inline]
    pub fn encode_int_3(&mut self, value: u32) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        let length = value.to_le_bytes();
        buf.extend_from_slice(&length[0..3]);
    }

    #[inline]
    pub fn encode_int_2(&mut self, value: u16) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.put_u16_le(value);
    }

    #[inline]
    pub fn encode_int_1(&mut self, value: u8) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.put_u8(value);
    }

    #[inline]
    pub fn encode_int_lenenc(&mut self, value: Option<&usize>) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        if let Some(value) = value {
            if *value > U24_MAX && *value <= std::u64::MAX as usize {
                buf.put_u8(0xFE);
                self.encode_int_8(*value as u64);
            } else if *value > std::u16::MAX as usize && *value <= U24_MAX {
                buf.put_u8(0xFD);
                self.encode_int_3(*value as u32);
            } else if *value > std::u8::MAX as usize && *value <= std::u16::MAX as usize {
                buf.put_u8(0xFC);
                self.encode_int_2( *value as u16);
            } else if *value <= std::u8::MAX as usize {
                buf.put_u8(0xFA);
                self.encode_int_1(*value as u8);
            } else {
                panic!("Value is too long");
            }
        } else {
            buf.put_u8(0xFB);
        }
    }

    #[inline]
    pub fn encode_string_lenenc(&mut self, string: &Bytes) {
        if string.len() > 0xFFF {
            panic!("String inside string lenenc serialization is too long");
        }

        self.encode_int_3(string.len() as u32);

        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };

        if string.len() > 0 {
            buf.extend_from_slice(string);
        }
    }

    #[inline]
    pub fn encode_string_null(&mut self, string: &Bytes) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.extend_from_slice(string);
        buf.put(0_u8);
    }

    #[inline]
    pub fn encode_string_fix(&mut self, bytes: &Bytes, size: usize) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        if size != bytes.len() {
            panic!("Sizes do not match");
        }

        buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_string_eof(&mut self, bytes: &Bytes) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_byte_lenenc(&mut self, bytes: &Bytes) {
        if bytes.len() > 0xFFF {
            panic!("String inside string lenenc serialization is too long");
        }

        self.encode_int_3(bytes.len() as u32);

        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };

        buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_byte_fix(&mut self, bytes: &Bytes, size: usize) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        if size != bytes.len() {
            panic!("Sizes do not match");
        }

        buf.extend_from_slice(bytes);
    }

    #[inline]
    pub fn encode_byte_eof(&mut self, bytes: &Bytes) {
        let buf = match self {
            Encoder::Ref { buf } => buf,
            Encoder::Owned { buf } => buf,
        };
        buf.extend_from_slice(bytes);
    }
}

impl<'a> From<&'a mut BytesMut> for Encoder<'a> {
    fn from(buf: &'a mut BytesMut) -> Encoder<'a> {
        Encoder::Ref {
            buf,
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
        let mut encoder = Encoder::new(&mut BytesMut::new());
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
