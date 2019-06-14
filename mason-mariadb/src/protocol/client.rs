// Reference: https://mariadb.com/kb/en/library/connection

use super::server::Capabilities;
use byteorder::ByteOrder;
use byteorder::LittleEndian;

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

impl Serialize for SSLRequestPacket {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // https://mariadb.com/kb/en/library/0-packet

        // Temporary storage for length: 3 bytes
        buf.push(0);
        buf.push(0);
        buf.push(0);

        // Sequence Numer
        buf.push(0);

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
