use bytes::{Buf, Bytes};
use bytestring::ByteString;
use sqlx_core::io::{BufExt, Deserialize};
use sqlx_core::Result;

use crate::io::MySqlBufExt;

// https://dev.mysql.com/doc/internals/en/packet-ERR_Packet.html
// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_err_packet.html
// https://mariadb.com/kb/en/err_packet/

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub(crate) struct ErrPacket {
    pub(crate) error_code: u16,
    pub(crate) sql_state: Option<ByteString>,
    pub(crate) error_message: ByteString,
}

impl ErrPacket {
    pub(crate) fn new(code: u16, message: &str) -> Self {
        let message = ByteString::from(message);
        let state = ByteString::from_static("HY000");

        Self { error_code: code, sql_state: Some(state), error_message: message }
    }
}

impl Deserialize<'_> for ErrPacket {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let tag = buf.get_u8();
        debug_assert!(tag == 0xff);

        let error_code = buf.get_u16_le();

        let sql_state = if buf[0] == b'#' {
            // if the next byte is '#' then we have the SQL STATE
            buf.advance(1);

            Some(buf.get_str(5)?)
        } else {
            None
        };

        let error_message = buf.get_str_eof()?;

        Ok(Self { sql_state, error_code, error_message })
    }
}

#[cfg(test)]
mod tests {
    use super::{Deserialize, ErrPacket};

    #[test]
    fn test_err_connect_auth() {
        const DATA: &[u8] = b"\xff\xe3\x04Client does not support authentication protocol requested by server; consider upgrading MySQL client";

        let ok = ErrPacket::deserialize(DATA.into()).unwrap();

        assert_eq!(ok.sql_state, None);
        assert_eq!(ok.error_code, 1251);
        assert_eq!(
            &ok.error_message,
            "Client does not support authentication protocol requested by server; consider upgrading MySQL client"
        );
    }

    #[test]
    fn test_err_out_of_order() {
        const DATA: &[u8] = b"\xff\x84\x04Got packets out of order";

        let ok = ErrPacket::deserialize(DATA.into()).unwrap();

        assert_eq!(ok.sql_state, None);
        assert_eq!(ok.error_code, 1156);
        assert_eq!(&ok.error_message, "Got packets out of order");
    }

    #[test]
    fn test_err_unknown_database() {
        const DATA: &[u8] = b"\xff\x19\x04#42000Unknown database \'unknown\'";

        let ok = ErrPacket::deserialize(DATA.into()).unwrap();

        assert_eq!(ok.sql_state.as_deref(), Some("42000"));
        assert_eq!(ok.error_code, 1049);
        assert_eq!(&ok.error_message, "Unknown database \'unknown\'");
    }
}
