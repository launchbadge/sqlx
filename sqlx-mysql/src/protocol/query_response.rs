use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use super::{Capabilities, ResultPacket};
use crate::io::MySqlBufExt;
use crate::MySqlDatabaseError;

/// The query-response packet is a meta-packet that starts with one of:
///
/// -   OK packet
/// -   ERR packet
/// -   LOCAL INFILE request (unimplemented)
/// -   Result Set
///
/// A result set is *also* a meta-packet that starts with a length-encoded
/// integer for the number of columns. That is all we return from this
/// deserialization and expect the executor to follow up with reading
/// more from the stream.
///
/// <https://dev.mysql.com/doc/internals/en/com-query-response.html>
///
#[derive(Debug)]
pub(crate) enum QueryResponse {
    End(ResultPacket),
    ResultSet { columns: u16 },
}

impl Deserialize<'_, Capabilities> for QueryResponse {
    fn deserialize_with(mut buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        // .get does not consume the byte
        match buf.get(0) {
            Some(0x00) | Some(0xff) => {
                ResultPacket::deserialize_with(buf, capabilities).map(Self::End)
            }

            // If its non-0, then its the number of columns and the start
            // of a result set
            Some(_) => {
                let columns = buf.get_uint_lenenc();

                // https://github.com/mysql/mysql-server/blob/8.0/sql/sql_const.h#L113
                // https://github.com/MariaDB/server/blob/b4fb15ccd4f2864483f8644c0236e63c814c8beb/sql/sql_const.h#L94
                debug_assert!(columns <= 4096);

                Ok(Self::ResultSet { columns: columns as u16 })
            }

            None => Err(MySqlDatabaseError::malformed_packet(
                "Received no bytes for COM_QUERY response",
            )
            .into()),
        }
    }
}
