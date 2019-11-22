use crate::{
    io::BufMut,
    mariadb::{
        io::BufMutExt,
        protocol::{Capabilities, Encode},
    },
};
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct HandshakeResponsePacket<'a> {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub client_collation: u8,
    pub username: &'a str,
    pub database: &'a str,
    pub auth_data: Option<&'a [u8]>,
    pub auth_plugin_name: Option<&'a str>,
    pub connection_attrs: &'a [(&'a str, &'a str)],
}

impl<'a> Encode for HandshakeResponsePacket<'a> {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities) {
        // client capabilities : int<4>
        buf.put_u32::<LittleEndian>(self.capabilities.bits() as u32);

        // max packet size : int<4>
        buf.put_u32::<LittleEndian>(self.max_packet_size);

        // client character collation : int<1>
        buf.put_u8(self.client_collation);

        // reserved : string<19>
        buf.advance(19);

        // if not (capabilities & CLIENT_MYSQL)
        if !capabilities.contains(Capabilities::CLIENT_MYSQL) {
            // extended client capabilities : int<4>
            buf.put_u32::<LittleEndian>((self.capabilities.bits() >> 32) as u32);
        } else {
            // reserved : int<4>
            buf.advance(4);
        }

        // username : string<NUL>
        buf.put_str_nul(self.username);

        // if (capabilities & PLUGIN_AUTH_LENENC_CLIENT_DATA)
        let auth_data = self.auth_data.unwrap_or_default();
        if capabilities.contains(Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA) {
            // authentication data : string<lenenc>
            buf.put_bytes_lenenc::<LittleEndian>(auth_data);
        } else if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            // length of authentication response : int<1>
            // authentication response (length is indicated by previous field) : string<fix>
            buf.put_u8(auth_data.len() as u8);
            buf.put_bytes(auth_data);
        } else {
            // 0x00 : int<1>
            buf.put_u8(0);
        }

        // if (capabilities & CLIENT_CONNECT_WITH_DB)
        if capabilities.contains(Capabilities::CONNECT_WITH_DB) {
            // default database name : string<NUL>
            buf.put_str_nul(self.database);
        }

        // if (capabilities & CLIENT_PLUGIN_AUTH)
        if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            // authentication plugin name : string<NUL>
            buf.put_str_nul(self.auth_plugin_name.unwrap_or_default());
        }

        // if (capabilities & CLIENT_CONNECT_ATTRS)
        if capabilities.contains(Capabilities::CONNECT_ATTRS) {
            // size of connection attributes : int<lenenc>
            buf.put_uint_lenenc::<LittleEndian, _>(self.connection_attrs.len() as u64);

            for (key, value) in self.connection_attrs {
                buf.put_str_lenenc::<LittleEndian>(key);
                buf.put_str_lenenc::<LittleEndian>(value);
            }
        }
    }
}
