use super::super::{
    client::TextProtocol, encode::Encoder, serialize::Serialize, types::Capabilities,
};
use failure::Error;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Serialize for ComProcessKill {
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComProcessKill.into());
        encoder.encode_int_4(self.process_id);

        Ok(())
    }
}
