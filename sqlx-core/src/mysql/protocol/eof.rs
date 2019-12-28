use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::io::BufExt;
use crate::mysql::protocol::{Capabilities, Decode, Status};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_eof_packet.html
// https://mariadb.com/kb/en/eof_packet/
#[derive(Debug)]
pub struct EofPacket {
    warnings: u16,
    status: Status,
}

impl Decode for EofPacket {
    fn decode(mut buf: &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let header = buf.get_u8()?;
        if header != 0xFE {
            return Err(protocol_err!(
                "expected EOF (0xFE); received 0x{:X}",
                header
            ))?;
        }

        let warnings = buf.get_u16::<LittleEndian>()?;
        let status = buf.get_u16::<LittleEndian>()?;

        Ok(Self {
            warnings,
            status: Status::from_bits_truncate(status),
        })
    }
}

//#[cfg(test)]
//mod tests {
//    use super::{Capabilities, Decode, ErrPacket, Status};
//
//    const ERR_HANDSHAKE_UNKNOWN_DB: &[u8] = b"\xff\x19\x04#42000Unknown database \'unknown\'";
//
//    #[test]
//    fn it_decodes_ok_handshake() {
//        let mut p = ErrPacket::decode(ERR_HANDSHAKE_UNKNOWN_DB).unwrap();
//
//        assert_eq!(p.error_code, 1049);
//        assert_eq!(&*p.sql_state, "42000");
//        assert_eq!(&*p.error_message, "Unknown database \'unknown\'");
//    }
//}
