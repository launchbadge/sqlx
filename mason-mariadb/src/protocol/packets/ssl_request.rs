use super::super::{encode::Encoder, serialize::Serialize, types::Capabilities};
use bytes::Bytes;
use failure::Error;

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

impl Serialize for SSLRequestPacket {
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

        Ok(())
    }
}
