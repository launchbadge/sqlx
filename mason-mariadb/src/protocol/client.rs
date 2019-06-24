// Reference: https://mariadb.com/kb/en/library/connection
// Packets: https://mariadb.com/kb/en/library/0-packet

// TODO: Handle lengths which are greater than 3 bytes
// Either break the backet into several smaller ones, or 
// return error
// TODO: Handle different Capabilities for server and client
// TODO: Handle when capability is set, but field is None

use super::server::Capabilities;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use bytes::Bytes;
use crate::protocol::serialize::*;

pub trait Serialize {
    fn serialize(&self, buf: &mut Vec<u8>);
}

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub server_capabilities: Capabilities,
    pub sequence_number: u8,
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

#[derive(Default, Debug)]
pub struct HandshakeResponsePacket {
    pub server_capabilities: Capabilities,
    pub sequence_number: u8,
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

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequestPacket {
    pub sequence_number: u8,
    pub auth_plugin_name: Bytes,
    pub auth_plugin_data: Bytes,
}

impl Serialize for SSLRequestPacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // Temporary storage for length: 3 bytes
        buf.write_u24::<LittleEndian>(0);
        // Sequence Number
        serialize_int_1(buf, self.sequence_number);

        // Packet body
        serialize_int_4(buf, self.capabilities.bits() as u32);
        serialize_int_4(buf, self.max_packet_size);
        serialize_int_1(buf, self.collation);

        // Filler
        serialize_byte_fix(buf, &Bytes::from_static(&[0u8; 19]), 19);

        if !(self.server_capabilities & Capabilities::CLIENT_MYSQL).is_empty() &&
            !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            if let Some(capabilities) = self.extended_capabilities {
                serialize_int_4(buf, capabilities.bits() as u32);
            }
        } else {
            serialize_byte_fix(buf, &Bytes::from_static(&[0u8;4]), 4);
        }

        // Set packet length
        serialize_length(buf);
    }
}

impl Serialize for HandshakeResponsePacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // Temporary storage for length: 3 bytes
        buf.write_u24::<LittleEndian>(0);
        // Sequence Number
        serialize_int_1(buf, self.sequence_number);

        // Packet body
        serialize_int_4(buf, self.capabilities.bits() as u32);
        serialize_int_4(buf, self.max_packet_size);
        serialize_int_1(buf, self.collation);

        // Filler
        serialize_byte_fix(buf, &Bytes::from_static(&[0u8; 19]), 19);

        if !(self.server_capabilities & Capabilities::CLIENT_MYSQL).is_empty() &&
            !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            if let Some(capabilities) = self.extended_capabilities {
                serialize_int_4(buf, capabilities.bits() as u32);
            }
        } else {
            serialize_byte_fix(buf, &Bytes::from_static(&[0u8;4]), 4);
        }

        serialize_string_null(buf, &self.username);

        if !(self.server_capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                serialize_string_lenenc(buf, &auth_data);
            }
        } else if !(self.server_capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                serialize_int_1(buf, self.auth_response_len.unwrap());
                serialize_string_fix(buf, &auth_response, self.auth_response_len.unwrap() as usize);
            }
        } else {
            serialize_int_1(buf, 0);
        }

        if !(self.server_capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                serialize_string_null(buf, &database);
            }
        }

        if !(self.server_capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                serialize_string_null(buf, &auth_plugin_name);
            }
        }

        if !(self.server_capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                serialize_int_lenenc(buf, Some(conn_attr_len));

                // Loop
                for (key, value) in conn_attr {
                    serialize_string_lenenc(buf, &key);
                    serialize_string_lenenc(buf, &value);
                }
            }
        }

        // Set packet length
        serialize_length(buf);
    }
}

impl Serialize for AuthenticationSwitchRequestPacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // Temporary storage for length: 3 bytes
        buf.write_u24::<LittleEndian>(0);
        // Sequence Number
        serialize_int_1(buf, self.sequence_number);

        // Packet body
        serialize_int_1(buf, 0xFE);
        serialize_string_null(buf, &self.auth_plugin_name);
        serialize_byte_eof(buf, &self.auth_plugin_data);

        // Set packet length
        serialize_length(buf);
    }
}
