// Reference: https://mariadb.com/kb/en/library/connection
// Packets: https://mariadb.com/kb/en/library/0-packet

// TODO: Handle lengths which are greater than 3 bytes
// Either break the backet into several smaller ones, or 
// return error
// TODO: Handle different Capabilities for server and client
// TODO: Handle when capability is set, but field is None

use super::server::Capabilities;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use bytes::Bytes;

pub trait Serialize {
    fn serialize(&self, buf: &mut Vec<u8>);
}

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub sequence_number: u8,
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

#[derive(Default, Debug)]
pub struct HandshakeResponsePacket {
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

impl Serialize for SSLRequestPacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // Temporary storage for length: 3 bytes
        buf.push(0);
        buf.push(0);
        buf.push(0);

        // Sequence Numer
        buf.push(self.sequence_number);

        LittleEndian::write_u32(buf, self.capabilities.bits() as u32);

        LittleEndian::write_u32(buf, self.max_packet_size);

        buf.push(self.collation);

        buf.extend_from_slice(&[0u8;19]);

        if !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            if let Some(capabilities) = self.extended_capabilities {
                LittleEndian::write_u32(buf, capabilities.bits() as u32);
            }
        } else {
            buf.extend_from_slice(&[0u8;4]);
        }
	
        // Get length in little endian bytes
        // packet length = byte[0] + (byte[1]<<8) + (byte[2]<<16)
        buf[0] = buf.len().to_le_bytes()[0];
        buf[1] = buf.len().to_le_bytes()[1];
        buf[2] = buf.len().to_le_bytes()[2];
    }
}

impl Serialize for HandshakeResponsePacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // Temporary storage for length: 3 bytes
        buf.push(0);
        buf.push(0);
        buf.push(0);

        // Sequence Numer
        buf.push(self.sequence_number);

        LittleEndian::write_u32(buf, self.capabilities.bits() as u32);

        LittleEndian::write_u32(buf, self.max_packet_size);

        buf.push(self.collation);

        buf.extend_from_slice(&[0u8;19]);

        if !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            if let Some(capabilities) = self.extended_capabilities {
                LittleEndian::write_u32(buf, capabilities.bits() as u32);
            }
        } else {
            buf.extend_from_slice(&[0u8;4]);
        }

        // Username: string<NUL>
        buf.extend_from_slice(&self.username);
        buf.push(0);

        if !(self.capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                // string<lenenc>
                buf.push(auth_data.len().to_le_bytes()[0]);
                buf.push(auth_data.len().to_le_bytes()[1]);
                buf.push(auth_data.len().to_le_bytes()[2]);
                buf.extend_from_slice(&auth_data);
            }
        } else if !(self.capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                buf.push(self.auth_response_len.unwrap());
                buf.extend_from_slice(&auth_response);
            }
        } else {
            buf.push(0);
        }

        if !(self.capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                buf.extend_from_slice(&database);
                buf.push(0);
            }
        }

        if !(self.capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                buf.extend_from_slice(&auth_plugin_name);
                buf.push(0);
            }
        }

        if !(self.capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                buf.push(conn_attr_len.to_le_bytes().len().to_le_bytes()[0]);
                buf.extend_from_slice(&conn_attr_len.to_le_bytes());

                // Loop
                for (key, value) in conn_attr {
                    // string<lenenc>
                    buf.push(key.len().to_le_bytes()[0]);
                    buf.push(key.len().to_le_bytes()[1]);
                    buf.push(key.len().to_le_bytes()[2]);
                    buf.extend_from_slice(&key);

                    // string<lenenc>
                    buf.push(value.len().to_le_bytes()[0]);
                    buf.push(value.len().to_le_bytes()[1]);
                    buf.push(value.len().to_le_bytes()[2]);
                    buf.extend_from_slice(&value);
                }
            }
        }

        // Get length in little endian bytes
        // packet length = byte[0] + (byte[1]<<8) + (byte[2]<<16)
        buf[0] = buf.len().to_le_bytes()[0];
        buf[1] = buf.len().to_le_bytes()[1];
        buf[2] = buf.len().to_le_bytes()[2];
    }
}
