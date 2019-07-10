use super::super::{serialize::Serialize, types::Capabilities};
use crate::connection::Connection;
use bytes::Bytes;
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
    fn serialize<'a, 'b>(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.encoder.encode_int_4(self.capabilities.bits() as u32);
        conn.encoder.encode_int_4(self.max_packet_size);
        conn.encoder.encode_int_1(self.collation);

        // Filler
        conn.encoder.encode_byte_fix(&Bytes::from_static(&[0u8; 19]), 19);

        if !(conn.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
            && !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
        {
            if let Some(capabilities) = self.extended_capabilities {
                conn.encoder.encode_int_4(capabilities.bits() as u32);
            }
        } else {
            conn.encoder.encode_byte_fix(&Bytes::from_static(&[0u8; 4]), 4);
        }

        conn.encoder.encode_string_null(&self.username);

        if !(conn.capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                conn.encoder.encode_string_lenenc(&auth_data);
            }
        } else if !(conn.capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                conn.encoder.encode_int_1(self.auth_response_len.unwrap());
                conn.encoder
                    .encode_string_fix(&auth_response, self.auth_response_len.unwrap() as usize);
            }
        } else {
            conn.encoder.encode_int_1(0);
        }

        if !(conn.capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                conn.encoder.encode_string_null(&database);
            }
        }

        if !(conn.capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                conn.encoder.encode_string_null(&auth_plugin_name);
            }
        }

        if !(conn.capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                conn.encoder.encode_int_lenenc(Some(conn_attr_len));

                // Loop
                for (key, value) in conn_attr {
                    conn.encoder.encode_string_lenenc(&key);
                    conn.encoder.encode_string_lenenc(&value);
                }
            }
        }

        Ok(())
    }
}
