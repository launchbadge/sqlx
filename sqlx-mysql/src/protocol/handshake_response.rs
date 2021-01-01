use bytes::BufMut;
use sqlx_core::io::{Serialize, WriteExt};
use sqlx_core::Result;

use crate::io::MySqlWriteExt;
use crate::protocol::Capabilities;

// https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::HandshakeResponse
// https://mariadb.com/kb/en/connection/#client-handshake-response

#[derive(Debug)]
pub(crate) struct HandshakeResponse<'a> {
    pub(crate) database: Option<&'a str>,
    pub(crate) max_packet_size: u32,
    pub(crate) charset: u8,
    pub(crate) username: Option<&'a str>,
    pub(crate) auth_plugin_name: Option<&'a str>,
    pub(crate) auth_response: Option<&'a [u8]>,
}

impl Serialize<'_, Capabilities> for HandshakeResponse<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, capabilities: Capabilities) -> Result<()> {
        buf.extend_from_slice(&(capabilities.bits() as u32).to_le_bytes());
        buf.extend_from_slice(&self.max_packet_size.to_le_bytes());
        buf.extend_from_slice(&self.charset.to_le_bytes());

        // reserved (all 0)
        buf.extend_from_slice(&[0_u8; 23]);

        buf.write_maybe_str_nul(self.username);

        let auth_response = self.auth_response.unwrap_or_default();

        if capabilities.contains(Capabilities::PLUGIN_AUTH_LENENC_DATA) {
            buf.write_bytes_lenenc(auth_response);
        } else if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            debug_assert!(auth_response.len() <= u8::max_value().into());

            buf.reserve(1 + auth_response.len());
            buf.push(auth_response.len() as u8);
            buf.extend_from_slice(auth_response);
        } else {
            buf.reserve(1 + auth_response.len());
            buf.extend_from_slice(auth_response);
            buf.push(b'\0');
        }

        if capabilities.contains(Capabilities::CONNECT_WITH_DB) {
            buf.write_maybe_str_nul(self.database);
        }

        if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            buf.write_maybe_str_nul(self.auth_plugin_name);
        }

        Ok(())
    }
}
