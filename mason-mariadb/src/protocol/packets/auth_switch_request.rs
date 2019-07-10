use super::super::{encode::Encoder, serialize::Serialize, types::Capabilities};
use bytes::Bytes;
use failure::Error;

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequestPacket {
    pub auth_plugin_name: Bytes,
    pub auth_plugin_data: Bytes,
}

impl Serialize for AuthenticationSwitchRequestPacket {
    fn serialize(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(0xFE);
        encoder.encode_string_null(&self.auth_plugin_name);
        encoder.encode_byte_eof(&self.auth_plugin_data);

        Ok(())
    }
}
