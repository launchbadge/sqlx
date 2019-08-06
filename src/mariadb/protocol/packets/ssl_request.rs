use bytes::Bytes;
use failure::Error;

use crate::mariadb::{BufMut, Capabilities, ConnContext, Connection, Encode};

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

impl Encode for SSLRequestPacket {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

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

        buf.put_length();

        Ok(())
    }
}
