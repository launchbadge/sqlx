use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use failure::Error;

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

impl Serialize for ComShutdown {
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComShutdown.into());
        encoder.encode_int_1(self.option.into());

        Ok(())
    }
}
