use super::super::{decode::Decoder, deserialize::Deserialize};
use failure::Error;
use crate::connection::Connection;

#[derive(Default, Debug)]
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<usize>,
}

impl Deserialize for ColumnPacket {
    fn deserialize(_conn: &mut Connection, decoder: &mut Decoder) -> Result<Self, Error> {
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();
        let columns = decoder.decode_int_lenenc();

        Ok(ColumnPacket { length, seq_no, columns })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use super::*;

    #[test]
    fn it_decodes_column_packet_0x_fb() -> Result<(), Error> {
        let buf = Bytes::from(b"\x01\0\0\x01\xFB".to_vec());
        let message = ColumnPacket::deserialize(&mut Connection::mock(), &mut Decoder::new(&buf))?;

        assert_eq!(message.columns, None);

        Ok(())
    }

    #[test]
    fn it_decodes_column_packet_0x_fd() -> Result<(), Error> {
        let buf = Bytes::from(b"\x04\0\0\x01\xFD\x01\x01\x01".to_vec());
        let message = ColumnPacket::deserialize(&mut Connection::mock(), &mut Decoder::new(&buf))?;

        assert_eq!(message.columns, Some(0x010101));

        Ok(())
    }

    #[test]
    fn it_fails_to_decode_column_packet_0x_fc() -> Result<(), Error> {
        let buf = Bytes::from(b"\x03\0\0\x01\xFC\x01\x01".to_vec());
        let message = ColumnPacket::deserialize(&mut Connection::mock(), &mut Decoder::new(&buf))?;

        assert_ne!(message.columns, Some(0x0100));

        Ok(())
    }
}

