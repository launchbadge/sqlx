use super::super::{encode::Encoder, serialize::Serialize, types::Capabilities};
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
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_4(self.capabilities.bits() as u32);
        encoder.encode_int_4(self.max_packet_size);
        encoder.encode_int_1(self.collation);

        // Filler
        encoder.encode_byte_fix(&Bytes::from_static(&[0u8; 19]), 19);

        if !(*server_capabilities & Capabilities::CLIENT_MYSQL).is_empty()
            && !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
        {
            if let Some(capabilities) = self.extended_capabilities {
                encoder.encode_int_4(capabilities.bits() as u32);
            }
        } else {
            encoder.encode_byte_fix(&Bytes::from_static(&[0u8; 4]), 4);
        }

        encoder.encode_string_null(&self.username);

        if !(*server_capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                encoder.encode_string_lenenc(&auth_data);
            }
        } else if !(*server_capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                encoder.encode_int_1(self.auth_response_len.unwrap());
                encoder.encode_string_fix(&auth_response, self.auth_response_len.unwrap() as usize);
            }
        } else {
            encoder.encode_int_1(0);
        }

        if !(*server_capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                encoder.encode_string_null(&database);
            }
        }

        if !(*server_capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                encoder.encode_string_null(&auth_plugin_name);
            }
        }

        if !(*server_capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                encoder.encode_int_lenenc(Some(conn_attr_len));

                // Loop
                for (key, value) in conn_attr {
                    encoder.encode_string_lenenc(&key);
                    encoder.encode_string_lenenc(&value);
                }
            }
        }

        Ok(())
    }
}
