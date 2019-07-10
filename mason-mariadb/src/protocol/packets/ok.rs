use super::super::{decode::Decoder, deserialize::Deserialize, types::ServerStatusFlag};
use bytes::Bytes;
use failure::Error;
use crate::connection::Connection;

#[derive(Default, Debug)]
pub struct OkPacket {
    pub length: u32,
    pub seq_no: u8,
    pub affected_rows: Option<usize>,
    pub last_insert_id: Option<usize>,
    pub server_status: ServerStatusFlag,
    pub warning_count: u16,
    pub info: Bytes,
    pub session_state_info: Option<Bytes>,
    pub value: Option<Bytes>,
}

impl Deserialize for OkPacket {
    fn deserialize(_conn: &mut Connection, decoder: &mut Decoder) -> Result<Self, Error> {
        // Packet header
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        // Packet body
        let packet_header = decoder.decode_int_1();
        if packet_header != 0 && packet_header != 0xFE {
            panic!("Packet header is not 0 or 0xFE for OkPacket");
        }

        let affected_rows = decoder.decode_int_lenenc();
        let last_insert_id = decoder.decode_int_lenenc();
        let server_status = ServerStatusFlag::from_bits_truncate(decoder.decode_int_2().into());
        let warning_count = decoder.decode_int_2();

        // Assuming CLIENT_SESSION_TRACK is unsupported
        let session_state_info = None;
        let value = None;

        let info = decoder.decode_byte_eof();

        Ok(OkPacket {
            length,
            seq_no,
            affected_rows,
            last_insert_id,
            server_status,
            warning_count,
            info,
            session_state_info,
            value,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn it_decodes_okpacket() -> Result<(), Error> {
        let buf = BytesMut::from(
            b"\
        \x0F\x00\x00\
        \x01\
        \x00\
        \xFB\
        \xFB\
        \x01\x01\
        \x00\x00\
        info\
        "
            .to_vec(),
        );

        let message = OkPacket::deserialize(&mut Connection::mock(), &mut Decoder::new(&buf.freeze()))?;

        assert_eq!(message.affected_rows, None);
        assert_eq!(message.last_insert_id, None);
        assert!(!(message.server_status & ServerStatusFlag::SERVER_STATUS_IN_TRANS).is_empty());
        assert_eq!(message.warning_count, 0);
        assert_eq!(message.info, b"info".to_vec());

        Ok(())
    }
}
