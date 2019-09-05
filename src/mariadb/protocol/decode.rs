use std::io;
use crate::mariadb::Capabilities;

// Decoding trait that is implemented by all packets
pub trait Decode<'a> {
    fn decode(src: &'a [u8], capabilities: Capabilities) -> io::Result<Self>
    where
        Self: Sized;
}
// A wrapper around a connection context to prevent
// deserializers from touching the stream, yet still have
// access to the connection context.
// Mainly used to simply to simplify number of parameters for deserializing functions
// pub struct DeContext<'a> {
//     pub ctx: &'a mut ConnContext,
//     pub stream: Option<&'a mut Framed>,
//     pub decoder: Decoder,
//     pub columns: Option<u64>,
//     pub column_defs: Option<Vec<ColumnDefPacket>>,
// }

// impl<'a> DeContext<'a> {
//     pub fn new(conn: &'a mut ConnContext, buf: Bytes) -> Self {
//         DeContext {
//             ctx: conn,
//             stream: None,
//             decoder: Decoder::new(buf),
//             columns: None,
//             column_defs: None,
//         }
//     }

//     pub fn with_stream(conn: &'a mut ConnContext, stream: &'a mut Framed) -> Self {
//         DeContext {
//             ctx: conn,
//             stream: Some(stream),
//             decoder: Decoder::new(Bytes::new()),
//             columns: None,
//             column_defs: None,
//         }
//     }

//     pub async fn next_packet(&mut self) -> Result<(), failure::Error> {
//         if let Some(stream) = &mut self.stream {
//             self.decoder = Decoder::new(stream.next_packet().await?);

//             return Ok(());
//         } else if self.decoder.buf.len() > 0 {
//             // There is still data in the buffer
//             return Ok(());
//         }

//         failure::bail!("Calling next_packet on DeContext with no stream provided")
//     }
// }

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
// impl Decoder {
//     // Create a new Decoder from an existing Bytes
//     pub fn new(buf: Bytes) -> Self {
//         Decoder { buf, index: 0 }
//     }

//     // Decode length from a packet
//     // Length is the first 3 bytes of the packet in little endian format
//     #[inline]
//     pub fn decode_length(&mut self) -> io::Resu Error> {
//         let length: u32 = (self.buf[self.index] as u32)
//             + ((self.buf[self.index + 1] as u32) << 8)
//             + ((self.buf[self.index + 2] as u32) << 16);
//         self.index += 3;

//         if self.buf.len() - self.index < length as usize {
//             return Err(err_msg("Lengths to do not match when decoding length"));
//         }

//         Ok(length)
//     }

//     #[inline]
//     pub fn decode_binary_decimal(&mut self) -> Bytes {
//         self.decode_string_lenenc()
//     }

//     #[inline]
//     pub fn decode_binary_double(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 8);
//         self.index += 8;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_bigint(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 8);
//         self.index += 8;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_int(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 4);
//         self.index += 4;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_mediumint(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 4);
//         self.index += 4;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_float(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 4);
//         self.index += 4;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_smallint(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 2);
//         self.index += 2;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_year(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 2);
//         self.index += 2;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_tinyint(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 1);
//         self.index += 1;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_date(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 5);
//         self.index += 5;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_timestamp(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 12);
//         self.index += 12;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_datetime(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 12);
//         self.index += 12;
//         value
//     }

//     #[inline]
//     pub fn decode_binary_time(&mut self) -> Bytes {
//         let value = self.buf.slice(self.index, self.index + 13);
//         self.index += 13;
//         value
//     }
// }

#[cfg(test)]
mod tests {
    use crate::__bytes_builder;
    use crate::mariadb::BufExt;
    use crate::io::Buf;
    use byteorder::LittleEndian;
    use std::io;

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
    fn it_decodes_int_lenenc_0x_fb() -> io::Result<()> {
        let buf = &__bytes_builder!(0xFB_u8)[..];
        let int = buf.get_uint_lenenc::<LittleEndian>()?;

        assert_eq!(int, None);

        Ok(())
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fc() -> io::Result<()> {
        let buf = &__bytes_builder!(0xFCu8, 1u8, 1u8)[..];
        let int = buf.get_uint_lenenc::<LittleEndian>()?;

        assert_eq!(int, Some(0x0101));

        Ok(())
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fd() -> io::Result<()> {
        let buf = &__bytes_builder!(0xFDu8, 1u8, 1u8, 1u8)[..];
        let int = buf.get_uint_lenenc::<LittleEndian>()?;

        assert_eq!(int, Some(0x010101));
        
        Ok(())
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fe() -> io::Result<()> {
        let buf = &__bytes_builder!(0xFE_u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8)[..];
        let int = buf.get_uint_lenenc::<LittleEndian>()?;

        assert_eq!(int, Some(0x0101010101010101));

        Ok(())
    }

    #[test]
    fn it_decodes_int_lenenc_0x_fa() -> io::Result<()> {
        let buf = &__bytes_builder!(0xFA_u8)[..];
        let int = buf.get_uint_lenenc::<LittleEndian>()?;

        assert_eq!(int, Some(0xFA));

        Ok(())
    }

    #[test]
    fn it_decodes_int_8() -> io::Result<()> {
        let buf = &__bytes_builder!(1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8)[..];
        let int = buf.get_u64::<LittleEndian>()?;

        assert_eq!(int, 0x0101010101010101);

        Ok(())
    }

    #[test]
    fn it_decodes_int_4() -> io::Result<()> {
        let buf = &__bytes_builder!(1u8, 1u8, 1u8, 1u8)[..];
        let int = buf.get_u32::<LittleEndian>()?;

        assert_eq!(int, 0x01010101);

        Ok(())
    }

    #[test]
    fn it_decodes_int_3() -> io::Result<()> {
        let buf = &__bytes_builder!(1u8, 1u8, 1u8)[..];
        let int = buf.get_u24::<LittleEndian>()?;

        assert_eq!(int, 0x010101);

        Ok(())
    }

    #[test]
    fn it_decodes_int_2() -> io::Result<()> {
        let buf = &__bytes_builder!(1u8, 1u8)[..];
        let int = buf.get_u16::<LittleEndian>()?;

        assert_eq!(int, 0x0101);

        Ok(())
    }

    #[test]
    fn it_decodes_int_1() -> io::Result<()> {
        let buf = &__bytes_builder!(1u8)[..];
        let int = buf.get_u8()?;

        assert_eq!(int, 1u8);

        Ok(())
    }

    #[test]
    fn it_decodes_string_lenenc() -> io::Result<()> {
        let buf = &&__bytes_builder!(3u8, b"sup")[..];
        let string = buf.get_str_lenenc()?;

        assert_eq!(string[..], b"sup"[..]);
        assert_eq!(string.len(), 3);

        Ok(())
    }

    #[test]
    fn it_decodes_string_fix() -> io::Result<()> {
        let buf = &__bytes_builder!(b"a")[..];
        let string = buf.get_str(1)?;

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);

        Ok(())
    }

    #[test]
    fn it_decodes_string_eof() -> io::Result<()> {
        let buf = &__bytes_builder!(b"a")[..];
        let string = buf.get_str_eof()?;

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);

        Ok(())
    }

    #[test]
    fn it_decodes_string_null() -> io::Result<()> {
        let buf = &__bytes_builder!(b"random\0", 1u8)[..];
        let string = buf.get_str_null()?;

        assert_eq!(&string[..], b"random");
        assert_eq!(string.len(), 6);

        Ok(())
    }

    #[test]
    fn it_decodes_byte_fix() -> io::Result<()> {
        let buf = &__bytes_builder!(b"a")[..];
        let string = buf.get_str(1)?;

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);

        Ok(())
    }

    #[test]
    fn it_decodes_byte_eof() -> io::Result<()> {
        let buf = &__bytes_builder!(b"a")[..];
        let string = buf.get_str_eof()?;

        assert_eq!(&string[..], b"a");
        assert_eq!(string.len(), 1);

        Ok(())
    }
}
