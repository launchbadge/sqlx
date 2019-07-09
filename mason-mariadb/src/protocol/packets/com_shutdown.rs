use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use bytes::BytesMut;
use failure::Error;

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

impl Serialize for ComShutdown {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComShutdown.into());
        encode_int_1(buf, self.option.into());

        Ok(())
    }
}
