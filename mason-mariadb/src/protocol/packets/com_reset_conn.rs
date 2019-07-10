use super::super::{
    client::TextProtocol, encode::Encoder, serialize::Serialize, types::Capabilities,
};
use failure::Error;

pub struct ComResetConnection();

impl Serialize for ComResetConnection {
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComResetConnection.into());

        Ok(())
    }
}
