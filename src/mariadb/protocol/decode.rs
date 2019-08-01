use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use failure::{err_msg, Error};

// Deserializing bytes and string do the same thing. Except that string also has a null terminated deserialzer
use super::packets::packet_header::PacketHeader;

// This is a simple wrapper around Bytes to make decoding easier
// since the index is always tracked
// The decoder is used to decode mysql protocol data-types
// into the appropriate Rust type or bytes::Bytes otherwise
// There are two types of protocols: Text and Binary.
// Text protocol is used for most things, and binary is used
// only for the results of prepared statements.
// MySql Text protocol data-types:
//      - byte<n> : Fixed-length bytes
//      - byte<lenenc> : Length-encoded bytes
//      - byte<EOF> : End-of-file length bytes
//      - int<n> : Fixed-length integers
//      - int<lenenc> : Length-encoded integers
//      - string<fix> : Fixed-length strings
//      - string<NUL> : Null-terminated strings
//      - string<lenenc> : Length-encoded strings
//      - string<EOF> : End-of-file length strings
// The decoder will decode all of the Text Protocol types, and if the data-type
// is of type int<*> then the decoder will convert that into the
// appropriate Rust type.
// The second protocol (Binary) protocol data-types (these rely on knowing the type from the column definition packet):
//      - DECIMAL : DECIMAL has no fixed size, so will be encoded as string<lenenc>.
//      - DOUBLE : DOUBLE is the IEEE 754 floating-point value in Little-endian format on 8 bytes.
//      - BIGINT : BIGINT is the value in Little-endian format on 8 bytes. Signed is defined by the Column field detail flag.
//      - INTEGER: INTEGER is the value in Little-endian format on 4 bytes. Signed is defined by the Column field detail flag.
//      - MEDIUMINT : MEDIUMINT is similar to INTEGER binary encoding, even if MEDIUM int is 3-bytes encoded server side. (Last byte will always be 0x00).
//      - FLOAT : FLOAT is the IEEE 754 floating-point value in Little-endian format on 4 bytes.
//      - SMALLINT : SMALLINT is the value in Little-endian format on 2 bytes. Signed is defined by the Column field detail flag.
//      - YEAR : YEAR uses the same format as SMALLINT.
//      - TINYINT : TINYINT is the value of 1 byte. Signed is defined by the Column field detail flag.
//      - DATE : Data is encoded in 5 bytes.
//          - First byte is the date length which must be 4
//          - Bytes 2-3 are the year on 2 bytes little-endian format
//          - Byte 4 is the month (1=january - 12=december)
//          - Byte 5 is the day of the month (0 - 31)
//      - TIMESTAMP: Data is encoded in 8 bytes without fractional seconds, 12 bytes with fractional seconds.
//          - Byte 1 is data length; 7 without fractional seconds, 11 with fractional seconds
//          - Bytes 2-3	are the year on 2 bytes little-endian format
//          - Byte 4 is the month (1=january - 12=december)
//          - Byte 5 is the day of the month (0 - 31)
//          - Byte 6 is the hour of day (0 if DATE type) (0-23)
//          - Byte 7 is the minutes (0 if DATE type) (0-59)
//          - Byte 8 is the seconds (0 if DATE type) (0-59)
//          - Bytes 9-12 is the micro-second on 4 bytes little-endian format (only if data-length is > 7) (0-9999)
//      - DATETIME : DATETIME uses the same format as TIMESTAMP binary encoding
//      - TIME : Data is encoded in 9 bytes without fractional seconds, 13 bytes with fractional seconds.
//          - Byte 1 is the data length; 8 without fractional seconds, 12 with fractional seconds
//          - Byte 2 determines negativity
//          - Bytes 3-6	are the date on 4 bytes little-endian format
//          - Byte 6 is the hour of day (0 if DATE type) (0-23)
//          - Byte 7 is the minutes (0 if DATE type) (0-59)
//          - Byte 8 is the seconds (0 if DATE type) (0-59)
//          - Bytes 10-13 are the micro-seconds on 4 bytes little-endian format (only if data-length is > 7)
pub struct Decoder {
    pub buf: Bytes,
    pub index: usize,
}

impl Decoder {
    // Create a new Decoder from an existing Bytes
    pub fn new(buf: Bytes) -> Self {
        Decoder { buf, index: 0 }
    }

    // Decode length from a packet
    // Length is the first 3 bytes of the packet in little endian format
    #[inline]
    pub fn decode_length(&mut self) -> Result<u32, Error> {
        let length: u32 = (self.buf[self.index] as u32) + ((self.buf[self.index + 1] as u32) << 8) + ((self.buf[self.index + 2] as u32) << 16);
        self.index += 3;

        if self.buf.len() - self.index < length as usize {
            return Err(err_msg("Lengths to do not match when decoding length"));
        }

        Ok(length)
    }

    // Helper method to get the tag of the packet. The tag is the 5th byte in the packet. It's not guaranteed
    // to exist or to be used for each packet. NOTE: Peeking at a tag DOES NOT increment index. This is used
    // to determine which type of packet was received before attempting to decode.
    #[inline]
    pub fn peek_tag(&self) -> Option<&u8> {
        if self.buf.len() < self.index + 4 {
            None
        } else {
            Some(&self.buf[self.index + 4])
        }
    }

    // Helper method to get the packet header. The packet header consist of the length (3 bytes) and
    // sequence number (1 byte). NOTE: Peeking a packet_header DOES NOT increment index. This is used
    // to determine if the packet is read to decode without starting the decoding process.
    #[inline]
    pub fn peek_packet_header(&self) -> Result<PacketHeader, Error> {
        let length: u32 = (self.buf[self.index] as u32) + ((self.buf[self.index + 1] as u32) << 8) + ((self.buf[self.index + 2] as u32) << 16);
        let seq_no = self.buf[self.index + 3];

        if self.buf.len() - self.index < length as usize {
            return Err(err_msg("Lengths to do not match when peeking header"));
        }

        Ok(PacketHeader { length, seq_no })
    }

    // Helper method to skip bytes via incrementing index. This is used because some packets have
    // "unused" bytes.
    #[inline]
    pub fn skip_bytes(&mut self, amount: usize) {
        self.index += amount;
    }

    // Deocde an int<lenenc> which is a length encoded int.
    // The first byte of the int<lenenc> determines the length of the int.
    // If the first byte is
    //      0xFB then the int is "NULL" or None in Rust terms.
    //      0xFC then the following 2 bytes are the int value i16.
    //      0xFD then the following 3 bytes are the int value i24.
    //      0xFE then the following 8 bytes are teh int value i64.
    //      0xFF then there was an error.
    // If the first byte is not in the previous list then that byte is the int value.
    #[inline]
    pub fn decode_int_lenenc_signed(&mut self) -> Option<i64> {
        match self.buf[self.index] {
            0xFB => {
                self.index += 1;
                None
            }
            0xFC => {
                let value = Some(LittleEndian::read_i16(&self.buf[self.index + 1..]) as i64);
                self.index += 3;
                value
            }
            0xFD => {
                let value = Some(LittleEndian::read_i24(&self.buf[self.index + 1..]) as i64);
                self.index += 4;
                value
            }
            0xFE => {
                let value = Some(LittleEndian::read_i64(&self.buf[self.index + 1..]) as i64);
                self.index += 9;
                value
            }
            0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
            _ => {
                let value = Some(self.buf[self.index] as i64);
                self.index += 1;
                value
            }
        }
    }

    // This is functionally identical to the previous method, but this one returns an u64 instead
    #[inline]
    pub fn decode_int_lenenc_unsigned(&mut self) -> Option<u64> {
        match self.buf[self.index] {
            0xFB => {
                self.index += 1;
                None
            }
            0xFC => {
                let value = Some(LittleEndian::read_u16(&self.buf[self.index + 1..]) as u64);
                self.index += 3;
                value
            }
            0xFD => {
                let value = Some(LittleEndian::read_u24(&self.buf[self.index + 1..]) as u64);
                self.index += 4;
                value
            }
            0xFE => {
                let value = Some(LittleEndian::read_u64(&self.buf[self.index + 1..]) as u64);
                self.index += 9;
                value
            }
            0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
            _ => {
                let value = Some(self.buf[self.index] as u64);
                self.index += 1;
                value
            }
        }
    }

    // Decode an int<8> which is a i64
    #[inline]
    pub fn decode_int_i64(&mut self) -> i64 {
        let value = LittleEndian::read_i64(&self.buf[self.index..]);
        self.index += 8;
        value
    }

    // Decode an int<4> which is a i32
    #[inline]
    pub fn decode_int_i32(&mut self) -> i32 {
        let value = LittleEndian::read_i32(&self.buf[self.index..]);
        self.index += 4;
        value
    }

    // Decode an int<4> which is a u32
    // This is a helper method for decoding flags.
    #[inline]
    pub fn decode_int_u32(&mut self) -> u32 {
        let value = LittleEndian::read_u32(&self.buf[self.index..]);
        self.index += 4;
        value
    }

    // Decode an int<3> which is a i24
    #[inline]
    pub fn decode_int_i24(&mut self) -> i32 {
        let value = LittleEndian::read_i24(&self.buf[self.index..]);
        self.index += 3;
        value
    }

    // Decode an int<2> which is a i16
    #[inline]
    pub fn decode_int_i16(&mut self) -> i16 {
        let value = LittleEndian::read_i16(&self.buf[self.index..]);
        self.index += 2;
        value
    }

    // Decode an int<2> as an u16
    // This is a helper method for decoding flags.
    #[inline]
    pub fn decode_int_u16(&mut self) -> u16 {
        let value = LittleEndian::read_u16(&self.buf[self.index..]);
        self.index += 2;
        value
    }

    // Decode an int<1> which is a u8
    #[inline]
    pub fn decode_int_u8(&mut self) -> u8 {
        let value = self.buf[self.index];
        self.index += 1;
        value
    }

    // Decode a string<lenenc> which is a length encoded string. First decode an int<lenenc> to get
    // the length of the string, and the the following n bytes are the contents.
    #[inline]
    pub fn decode_string_lenenc(&mut self) -> Bytes {
        let length = self.decode_int_lenenc_unsigned().unwrap_or(0);
        let value = self.buf.slice(self.index, self.index + length as usize);
        self.index = self.index + length as usize;
        value
    }

    // Decode a string<fix> which is a string of fixed length.
    #[inline]
    pub fn decode_string_fix(&mut self, length: usize) -> Bytes {
        let value = self.buf.slice(self.index, self.index + length);
        self.index = self.index + length as usize;
        value
    }

    // Decode a string<eof> which is a string which is terminated byte the end of the packet.
    #[inline]
    pub fn decode_string_eof(&mut self, length: Option<usize>) -> Bytes {
        let value = self.buf.slice(self.index, if let Some(len) = length {
            if len >= self.index {
                len
            } else {
                self.buf.len()
            }
        } else {
            self.buf.len()
        });
        self.index = self.buf.len();
        value
    }

    // Decode a string<null> which is a null terminated string (C style string).
    #[inline]
    pub fn decode_string_null(&mut self) -> Result<Bytes, Error> {
        if let Some(null_index) = memchr::memchr(0, &self.buf[self.index..]) {
            let value = self.buf.slice(self.index, self.index + null_index);
            self.index = self.index + null_index + 1;
            Ok(value)
        } else {
            Err(err_msg("Null index no found"))
        }
    }

    // Same as the string counter part, but copied to maintain consistency with the spec.
    #[inline]
    pub fn decode_byte_fix(&mut self, length: usize) -> Bytes {
        let value = self.buf.slice(self.index, self.index + length);
        self.index = self.index + length as usize;
        value
    }

    // Same as the string counter part, but copied to maintain consistency with the spec.
    #[inline]
    pub fn decode_byte_lenenc(&mut self) -> Bytes {
        let length = self.decode_int_u8();
        let value = self.buf.slice(self.index, self.index + length as usize);
        self.index = self.index + length as usize;
        value
    }

    // Same as the string counter part, but copied to maintain consistency with the spec.
    #[inline]
    pub fn decode_byte_eof(&mut self, length: Option<usize>) -> Bytes {
        let value = self.buf.slice(self.index, if let Some(len) = length {
            if len >= self.index {
                len
            } else {
                self.buf.len()
            }
        } else {
            self.buf.len()
        });
        self.index = self.buf.len();
        value
    }

    #[inline]
    pub fn decode_binary_decimal(&mut self) -> Bytes {
        self.decode_string_lenenc()
    }

    #[inline]
    pub fn decode_binary_double(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 8);
        self.index += 8;
        value
    }

    #[inline]
    pub fn decode_binary_bigint(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 8);
        self.index += 8;
        value
    }

    #[inline]
    pub fn decode_binary_int(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 4);
        self.index += 4;
        value
    }

    #[inline]
    pub fn decode_binary_mediumint(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 4);
        self.index += 4;
        value
    }

    #[inline]
    pub fn decode_binary_float(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 4);
        self.index += 4;
        value
    }

    #[inline]
    pub fn decode_binary_smallint(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 2);
        self.index += 2;
        value
    }

    #[inline]
    pub fn decode_binary_year(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 2);
        self.index += 2;
        value
    }

    #[inline]
    pub fn decode_binary_tinyint(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 1);
        self.index += 1;
        value
    }

    #[inline]
    pub fn decode_binary_date(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 5);
        self.index += 5;
        value
    }

    #[inline]
    pub fn decode_binary_timestamp(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 12);
        self.index += 12;
        value
    }

    #[inline]
    pub fn decode_binary_datetime(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 12);
        self.index += 12;
        value
    }

    #[inline]
    pub fn decode_binary_time(&mut self) -> Bytes {
        let value = self.buf.slice(self.index, self.index + 13);
        self.index += 13;
        value

    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use failure::Error;
    use crate::__bytes_builder;

    use super::*;

    // [X] it_decodes_int_lenenc
    // [X] it_decodes_int_8
    // [X] it_decodes_int_4
    // [X] it_decodes_int_3
    // [X] it_decodes_int_2
    // [X] it_decodes_int_1
    // [X] it_decodes_string_lenenc
    // [X] it_decodes_string_fix
    // [X] it_decodes_string_eof
    // [X] it_decodes_string_null
    // [X] it_decodes_byte_lenenc
    // [X] it_decodes_byte_eof

    #[test]
    fn it_decodes_int_lenenc_0x_fb() {
        let buf = __bytes_builder!(0xFB_u8);
        let mut decoder = Decoder::new(buf);
        let int = decoder.decode_int_lenenc_unsigned();

        assert_eq!(int, None);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fc() {
        let buf =__bytes_builder!(0xFCu8, 1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int = decoder.decode_int_lenenc_unsigned();

        assert_eq!(int, Some(0x0101));
        assert_eq!(decoder.index, 3);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fd() {
        let buf = __bytes_builder!(0xFDu8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int = decoder.decode_int_lenenc_unsigned();

        assert_eq!(int, Some(0x010101));
        assert_eq!(decoder.index, 4);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fe() {
        let buf = __bytes_builder!(0xFE_u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int = decoder.decode_int_lenenc_unsigned();

        assert_eq!(int, Some(0x0101010101010101));
        assert_eq!(decoder.index, 9);
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fa() {
        let buf = __bytes_builder!(0xFA_u8);
        let mut decoder = Decoder::new(buf);
        let int = decoder.decode_int_lenenc_unsigned();

        assert_eq!(int, Some(0xFA));
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_int_8() {
        let buf = __bytes_builder!(1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int: i64 = decoder.decode_int_i64();

        assert_eq!(int, 0x0101010101010101);
        assert_eq!(decoder.index, 8);
    }

    #[test]
    fn it_decodes_int_4() {
        let buf = __bytes_builder!(1u8, 1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int: i32 = decoder.decode_int_i32();

        assert_eq!(int, 0x01010101);
        assert_eq!(decoder.index, 4);
    }

    #[test]
    fn it_decodes_int_3() {
        let buf = __bytes_builder!(1u8, 1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int: i32 = decoder.decode_int_i24();

        assert_eq!(int, 0x010101);
        assert_eq!(decoder.index, 3);
    }

    #[test]
    fn it_decodes_int_2() {
        let buf = __bytes_builder!(1u8, 1u8);
        let mut decoder = Decoder::new(buf);
        let int: i16 = decoder.decode_int_i16();

        assert_eq!(int, 0x0101);
        assert_eq!(decoder.index, 2);
    }

    #[test]
    fn it_decodes_int_1() {
        let buf = __bytes_builder!(1u8);
        let mut decoder = Decoder::new(buf);
        let int: u8 = decoder.decode_int_u8();

        assert_eq!(int, 1u8);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_string_lenenc() {
        let buf = __bytes_builder!(3u8, b"sup");
        let mut decoder = Decoder::new(buf);
        let string: Bytes = decoder.decode_string_lenenc();

        assert_eq!(string[..], b"sup"[..]);
        assert_eq!(string.len(), 3);
        assert_eq!(decoder.index, 4);
    }

    #[test]
    fn it_decodes_string_fix() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(buf);
        let string: Bytes = decoder.decode_string_fix(1);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_string_eof() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(buf);
        let string: Bytes = decoder.decode_string_eof(None);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_string_null() -> Result<(), Error> {
        let buf = __bytes_builder!(b"random\0", 1u8);
        let mut decoder = Decoder::new(buf);
        let string: Bytes = decoder.decode_string_null()?;

        assert_eq!(&string[..], b"random");

        assert_eq!(string.len(), 6);
        // Skips null byte
        assert_eq!(decoder.index, 7);

        Ok(())
    }

    #[test]
    fn it_decodes_byte_fix() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(buf);
        let string: Bytes = decoder.decode_byte_fix(1);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }

    #[test]
    fn it_decodes_byte_eof() {
        let buf = __bytes_builder!(b"a");
        let mut decoder = Decoder::new(buf);
        let string: Bytes = decoder.decode_byte_eof(None);

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);
        assert_eq!(decoder.index, 1);
    }
}
