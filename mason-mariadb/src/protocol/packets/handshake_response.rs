use super::super::{encode::*, serialize::Serialize, types::Capabilities};
use bytes::{Bytes, BytesMut};
use failure::Error;

#[derive(Default, Debug)]
pub struct HandshakeResponsePacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
    pub username: Bytes,
    pub auth_data: Option<Bytes>,
    pub auth_response_len: Option<u8>,
    pub auth_response: Option<Bytes>,
    pub database: Option<Bytes>,
    pub auth_plugin_name: Option<Bytes>,
    pub conn_attr_len: Option<usize>,
    pub conn_attr: Option<Vec<(Bytes, Bytes)>>,
}

impl Serialize for HandshakeResponsePacket {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_4(buf, self.capabilities.bits() as u32);
        encode_int_4(buf, self.max_packet_size);
        encode_int_1(buf, self.collation);

        // Filler
        encode_byte_fix(buf, &Bytes::from_static(&[0u8; 19]), 19);

        if !(*server_capabilities & Capabilities::CLIENT_MYSQL).is_empty()
            && !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
        {
            if let Some(capabilities) = self.extended_capabilities {
                encode_int_4(buf, capabilities.bits() as u32);
            }
        } else {
            encode_byte_fix(buf, &Bytes::from_static(&[0u8; 4]), 4);
        }

        encode_string_null(buf, &self.username);

        if !(*server_capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                encode_string_lenenc(buf, &auth_data);
            }
        } else if !(*server_capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                encode_int_1(buf, self.auth_response_len.unwrap());
                encode_string_fix(buf, &auth_response, self.auth_response_len.unwrap() as usize);
            }
        } else {
            encode_int_1(buf, 0);
        }

        if !(*server_capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                encode_string_null(buf, &database);
            }
        }

        if !(*server_capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                encode_string_null(buf, &auth_plugin_name);
            }
        }

        if !(*server_capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                encode_int_lenenc(buf, Some(conn_attr_len));

                // Loop
                for (key, value) in conn_attr {
                    encode_string_lenenc(buf, &key);
                    encode_string_lenenc(buf, &value);
                }
            }
        }

        Ok(())
    }
}
