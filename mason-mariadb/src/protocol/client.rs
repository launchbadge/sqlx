// Reference: https://mariadb.com/kb/en/library/connection

use super::server::Capabilities;
use byteorder::ByteOrder;
use byteorder::LittleEndian;

pub trait Serialize {
    fn serialize(&self, buf: &mut Vec<u8>);
}

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

impl Serialize for SSLRequestPacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // FIXME: Prepend length of packet in standard packet form
        // https://mariadb.com/kb/en/library/0-packet
        // buf.push(32);
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
    }
}
