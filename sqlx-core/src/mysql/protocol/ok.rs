use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::io::BufExt;
use crate::mysql::protocol::Status;
use crate::mysql::MySql;

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_ok_packet.html
// https://mariadb.com/kb/en/ok_packet/
#[derive(Debug)]
pub(crate) struct OkPacket {
    pub(crate) affected_rows: u64,
    pub(crate) last_insert_id: u64,
    pub(crate) status: Status,
    pub(crate) warnings: u16,
    pub(crate) info: Box<str>,
}

impl OkPacket {
    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<MySql, Self>
    where
        Self: Sized,
    {
        let header = buf.get_u8()?;
        if header != 0 && header != 0xFE {
            return Err(protocol_err!(
                "expected 0x00 or 0xFE; received 0x{:X}",
                header
            ))?;
        }

        let affected_rows = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0); // 0
        let last_insert_id = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0); // 2
        let status = Status::from_bits_truncate(buf.get_u16::<LittleEndian>()?); //
        let warnings = buf.get_u16::<LittleEndian>()?;
        let info = buf.get_str(buf.len())?.into();

        Ok(Self {
            affected_rows,
            last_insert_id,
            status,
            warnings,
            info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{OkPacket, Status};

    const OK_HANDSHAKE: &[u8] = b"\x00\x00\x00\x02@\x00\x00";

    #[test]
    fn it_decodes_ok_handshake() {
        let p = OkPacket::read(OK_HANDSHAKE).unwrap();

        assert_eq!(p.affected_rows, 0);
        assert_eq!(p.last_insert_id, 0);
        assert_eq!(p.warnings, 0);
        assert!(p.status.contains(Status::SERVER_STATUS_AUTOCOMMIT));
        assert!(p.status.contains(Status::SERVER_SESSION_STATE_CHANGED));
        assert!(p.info.is_empty());
    }
}
