use crate::{
    io::BufMut,
    mysql::{
        io::BufMutExt,
        protocol::{Capabilities, Encode},
    },
};
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct SslRequest {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub client_collation: u8,
}

impl Encode for SslRequest {
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
    }
}
