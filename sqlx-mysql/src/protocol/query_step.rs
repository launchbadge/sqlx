use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::{Error, Result};

use super::{Capabilities, ResultPacket};
use crate::protocol::Packet;
use crate::MySqlDatabaseError;

/// <https://dev.mysql.com/doc/internals/en/com-query-response.html#packet-ProtocolText::Resultset>
/// <https://mariadb.com/kb/en/result-set-packets/>
#[derive(Debug)]
pub(crate) enum QueryStep {
    Row(Packet),
    End(ResultPacket),
}

impl Deserialize<'_, Capabilities> for QueryStep {
    fn deserialize_with(buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        // .get does not consume the byte
        match buf.get(0) {
            // To safely confirm that a packet with a 0xFE header is an OK packet (OK_Packet) or an
            // EOF packet (EOF_Packet), you must also check that the packet length is less than 0xFFFFFF
            Some(0xfe) if buf.len() < 0xFF_FF_FF => {
                ResultPacket::deserialize_with(buf, capabilities).map(Self::End)
            }

            // ERR packets are handled on a higher-level (in `recv_packet`), we will
            // never receive them here

            // If its non-0, then its a Row
            Some(_) => Ok(Self::Row(Packet { bytes: buf })),

            None => Err(Error::connect(MySqlDatabaseError::malformed_packet(
                "Received no bytes for the next step in a result set",
            ))),
        }
    }
}
