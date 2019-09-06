use crate::{
    io::Buf,
    mariadb::{
        io::BufExt,
        protocol::{Capabilities, ServerStatusFlag},
    },
};
use byteorder::LittleEndian;
use std::io;

// https://mariadb.com/kb/en/library/ok_packet/
#[derive(Debug)]
pub struct OkPacket {
    pub affected_rows: u64,
    pub last_insert_id: u64,
    pub server_status: ServerStatusFlag,
    pub warning_count: u16,
    pub info: Box<str>,
    pub session_state_info: Option<Box<[u8]>>,
    pub value_of_variable: Option<Box<str>>,
}

impl OkPacket {
    fn decode(mut buf: &[u8], capabilities: Capabilities) -> io::Result<Self> {
        let header = buf.get_u8()?;
        if header != 0 && header != 0xFE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected 0x00 or 0xFE; received 0x{:X}", header),
            ));
        }

        let affected_rows = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0);
        let last_insert_id = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0);
        let server_status = ServerStatusFlag::from_bits_truncate(buf.get_u16::<LittleEndian>()?);
        let warning_count = buf.get_u16::<LittleEndian>()?;

        let info;
        let mut session_state_info = None;
        let mut value_of_variable = None;

        if capabilities.contains(Capabilities::CLIENT_SESSION_TRACK) {
            info = buf
                .get_str_lenenc::<LittleEndian>()?
                .unwrap_or_default()
                .to_owned()
                .into();
            session_state_info = buf.get_byte_lenenc::<LittleEndian>()?.map(Into::into);
            value_of_variable = buf.get_str_lenenc::<LittleEndian>()?.map(Into::into);
        } else {
            info = buf.get_str_eof()?.to_owned().into();
        }

        Ok(Self {
            affected_rows,
            last_insert_id,
            server_status,
            warning_count,
            info,
            session_state_info,
            value_of_variable,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_ok_packet() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // 0x00 : OK_Packet header or (0xFE if CLIENT_DEPRECATE_EOF is set)
            0u8,
            // int<lenenc> affected rows
            0xFB_u8,
            // int<lenenc> last insert id
            0xFB_u8,
            // int<2> server status
            1u8, 1u8,
            // int<2> warning count
            0u8, 0u8,
            // if session_tracking_supported (see CLIENT_SESSION_TRACK) {
            //   string<lenenc> info
            //   if (status flags & SERVER_SESSION_STATE_CHANGED) {
            //     string<lenenc> session state info
            //     string<lenenc> value of variable
            //   }
            // } else {
            //   string<EOF> info
                b"info"
            // }
        );

        let message = OkPacket::decode(&buf, Capabilities::empty())?;

        assert_eq!(message.affected_rows, 0);
        assert_eq!(message.last_insert_id, 0);
        assert!(
            message
                .server_status
                .contains(ServerStatusFlag::SERVER_STATUS_IN_TRANS)
        );
        assert_eq!(message.warning_count, 0);
        assert_eq!(message.info, "info".into());

        Ok(())
    }
}
