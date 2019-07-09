use super::super::{encode::*, serialize::Serialize, types::Capabilities};
use bytes::{Bytes, BytesMut};
use failure::Error;

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

impl Serialize for SSLRequestPacket {
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

        Ok(())
    }
}
