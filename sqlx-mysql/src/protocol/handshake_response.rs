use sqlx_core::io::{Serialize, WriteExt};
use sqlx_core::Result;

use crate::io::MySqlWriteExt;
use crate::protocol::{Capabilities, MaybeCommand};

// https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::HandshakeResponse
// https://mariadb.com/kb/en/connection/#client-handshake-response

#[derive(Debug)]
pub(crate) struct HandshakeResponse<'a> {
    pub(crate) database: Option<&'a str>,
    pub(crate) max_packet_size: u32,
    pub(crate) charset: u8,
    pub(crate) username: Option<&'a str>,
    pub(crate) auth_plugin_name: &'a str,
    pub(crate) auth_response: Vec<u8>,
}

impl MaybeCommand for HandshakeResponse<'_> {}

impl Serialize<'_, Capabilities> for HandshakeResponse<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, capabilities: Capabilities) -> Result<()> {
        // the truncation is the intent
        // capability bits over 32 are MariaDB only (and we don't currently support them)
        #[allow(clippy::cast_possible_truncation)]
        buf.extend_from_slice(&(capabilities.bits() as u32).to_le_bytes());
        buf.extend_from_slice(&self.max_packet_size.to_le_bytes());
        buf.push(self.charset);

        // reserved (all 0)
        buf.extend_from_slice(&[0_u8; 23]);

        buf.write_maybe_str_nul(self.username);

        let auth_response = self.auth_response.as_slice();

        if capabilities.contains(Capabilities::PLUGIN_AUTH_LENENC_DATA) {
            buf.write_bytes_lenenc(auth_response);
        } else if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            debug_assert!(auth_response.len() <= u8::max_value().into());

            buf.reserve(1 + auth_response.len());

            // in debug mode, we assert that the auth response is not too big
            #[allow(clippy::cast_possible_truncation)]
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
            buf.write_str_nul(self.auth_plugin_name);
        }

        Ok(())
    }
}
