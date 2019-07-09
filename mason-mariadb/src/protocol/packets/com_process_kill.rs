use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use bytes::BytesMut;
use failure::Error;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Serialize for ComProcessKill {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComProcessKill.into());
        encode_int_4(buf, self.process_id);

        Ok(())
    }
}
