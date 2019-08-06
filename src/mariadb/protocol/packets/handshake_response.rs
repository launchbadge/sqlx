use crate::mariadb::{BufMut, Capabilities, ConnContext, Connection, Encode};
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

impl Encode for HandshakeResponsePacket {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(1);

        buf.put_int_u32(self.capabilities.bits() as u32);
        buf.put_int_u32(self.max_packet_size);
        buf.put_int_u8(self.collation);

        // Filler
        buf.put_byte_fix(&Bytes::from_static(&[0u8; 19]), 19);

        if !(ctx.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
            && !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
        {
            if let Some(capabilities) = self.extended_capabilities {
                buf.put_int_u32(capabilities.bits() as u32);
            }
        } else {
            buf.put_byte_fix(&Bytes::from_static(&[0u8; 4]), 4);
        }

        buf.put_string_null(&self.username);

        if !(ctx.capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                buf.put_string_lenenc(&auth_data);
            }
        } else if !(ctx.capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                buf.put_int_u8(self.auth_response_len.unwrap());
                buf.put_string_fix(&auth_response, self.auth_response_len.unwrap() as usize);
            }
        } else {
            buf.put_int_u8(0);
        }

        if !(ctx.capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                buf.put_string_null(&database);
            }
        }

        if !(ctx.capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                buf.put_string_null(&auth_plugin_name);
            }
        }

        if !(ctx.capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                buf.put_int_lenenc(Some(conn_attr_len));

                // Loop
                for (key, value) in conn_attr {
                    buf.put_string_lenenc(&key);
                    buf.put_string_lenenc(&value);
                }
            }
        }

        buf.put_length();

        Ok(())
    }
}
