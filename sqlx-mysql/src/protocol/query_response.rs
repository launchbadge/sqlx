use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::{Error, Result};

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
    ResultSet { columns: u64 },
}

impl Deserialize<'_, Capabilities> for QueryResponse {
    fn deserialize_with(mut buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        // .get does not consume the byte
        match buf.get(0) {
            Some(0x00) => ResultPacket::deserialize_with(buf, capabilities).map(Self::End),

            // ERR packets are handled on a higher-level (in `recv_packet`), we will
            // never receive them here

            // If its non-0, then its the number of columns and the start
            // of a result set
            Some(_) => Ok(Self::ResultSet { columns: buf.get_uint_lenenc() }),

            None => Err(Error::connect(MySqlDatabaseError::malformed_packet(
                "Received no bytes for COM_QUERY response",
            ))),
        }
    }
}
