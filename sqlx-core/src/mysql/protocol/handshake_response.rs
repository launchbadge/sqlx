use byteorder::LittleEndian;

use crate::io::BufMut;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::{AuthPlugin, Capabilities, Encode};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase_packets_protocol_handshake_response.html
// https://mariadb.com/kb/en/connection/#handshake-response-packet
#[derive(Debug)]
pub struct HandshakeResponse<'a> {
    pub max_packet_size: u32,
    pub client_collation: u8,
    pub username: &'a str,
    pub database: Option<&'a str>,
    pub auth_plugin: &'a AuthPlugin,
    pub auth_response: &'a [u8],
}

impl Encode for HandshakeResponse<'_> {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities) {
        // client capabilities : int<4>
        buf.put_u32::<LittleEndian>(capabilities.bits() as u32);

        // max packet size : int<4>
        buf.put_u32::<LittleEndian>(self.max_packet_size);

        // client character collation : int<1>
        buf.put_u8(self.client_collation);

        // reserved : string<19>
        buf.advance(19);

        if capabilities.contains(Capabilities::MYSQL) {
            // reserved : string<4>
            buf.advance(4);
        } else {
            // extended client capabilities : int<4>
            buf.put_u32::<LittleEndian>((capabilities.bits() >> 32) as u32);
        }

        // username : string<NUL>
        buf.put_str_nul(self.username);

        if capabilities.contains(Capabilities::PLUGIN_AUTH_LENENC_DATA) {
            // auth_response : string<lenenc>
            buf.put_bytes_lenenc::<LittleEndian>(self.auth_response);
        } else if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            let auth_response = self.auth_response;

            // auth_response_length : int<1>
            buf.put_u8(auth_response.len() as u8);

            // auth_response : string<{auth_response_length}>
            buf.put_bytes(auth_response);
        } else {
            // no auth : int<1>
            buf.put_u8(0);
        }

        if capabilities.contains(Capabilities::CONNECT_WITH_DB) {
            if let Some(database) = self.database {
                // database : string<NUL>
                buf.put_str_nul(database);
            }
        }

        if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            // client_plugin_name : string<NUL>
            buf.put_str_nul(self.auth_plugin.as_str());
        }
    }
}
