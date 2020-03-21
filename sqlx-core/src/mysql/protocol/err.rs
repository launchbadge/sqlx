use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::protocol::Capabilities;
use crate::mysql::MySql;

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_err_packet.html
// https://mariadb.com/kb/en/err_packet/
#[derive(Debug)]
pub struct ErrPacket {
    pub error_code: u16,
    pub sql_state: Option<Box<str>>,
    pub error_message: Box<str>,
}

impl ErrPacket {
    pub(crate) fn read(mut buf: &[u8], capabilities: Capabilities) -> crate::Result<MySql, Self>
    where
        Self: Sized,
    {
        let header = buf.get_u8()?;
        if header != 0xFF {
            return Err(protocol_err!(
                "expected 0xFF for ERR_PACKET; received 0x{:X}",
                header
            ))?;
        }

        let error_code = buf.get_u16::<LittleEndian>()?;

        let mut sql_state = None;

        if capabilities.contains(Capabilities::PROTOCOL_41) {
            // If the next byte is '#' then we have a SQL STATE
            if buf.get(0) == Some(&0x23) {
                buf.advance(1);
                sql_state = Some(buf.get_str(5)?.into())
            }
        }

        let error_message = buf.get_str(buf.len())?.into();

        Ok(Self {
            error_code,
            sql_state,
            error_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Capabilities, ErrPacket};

    const ERR_PACKETS_OUT_OF_ORDER: &[u8] = b"\xff\x84\x04Got packets out of order";

    const ERR_HANDSHAKE_UNKNOWN_DB: &[u8] = b"\xff\x19\x04#42000Unknown database \'unknown\'";

    #[test]
    fn it_decodes_packets_out_of_order() {
        let p = ErrPacket::read(ERR_PACKETS_OUT_OF_ORDER, Capabilities::PROTOCOL_41).unwrap();

        assert_eq!(&*p.error_message, "Got packets out of order");
        assert_eq!(p.error_code, 1156);
        assert_eq!(p.sql_state, None);
    }

    #[test]
    fn it_decodes_ok_handshake() {
        let p = ErrPacket::read(ERR_HANDSHAKE_UNKNOWN_DB, Capabilities::PROTOCOL_41).unwrap();

        assert_eq!(p.error_code, 1049);
        assert_eq!(p.sql_state.as_deref(), Some("42000"));
        assert_eq!(&*p.error_message, "Unknown database \'unknown\'");
    }
}
