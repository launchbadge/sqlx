use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::io::BufExt;
use crate::mysql::protocol::{Capabilities, Decode, Status};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_err_packet.html
// https://mariadb.com/kb/en/err_packet/
#[derive(Debug)]
pub struct ErrPacket {
    pub error_code: u16,
    pub sql_state: Box<str>,
    pub error_message: Box<str>,
}

impl Decode for ErrPacket {
    fn decode(mut buf: &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let header = buf.get_u8()?;
        if header != 0xFF {
            return Err(protocol_err!("expected 0xFF; received 0x{:X}", header))?;
        }

        let error_code = buf.get_u16::<LittleEndian>()?;

        let _sql_state_marker: u8 = buf.get_u8()?;
        let sql_state = buf.get_str(5)?.into();

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
    use super::{Capabilities, Decode, ErrPacket, Status};

    const ERR_HANDSHAKE_UNKNOWN_DB: &[u8] = b"\xff\x19\x04#42000Unknown database \'unknown\'";

    #[test]
    fn it_decodes_ok_handshake() {
        let mut p = ErrPacket::decode(ERR_HANDSHAKE_UNKNOWN_DB).unwrap();

        assert_eq!(p.error_code, 1049);
        assert_eq!(&*p.sql_state, "42000");
        assert_eq!(&*p.error_message, "Unknown database \'unknown\'");
    }
}
