use super::super::{serialize::Serialize, types::Capabilities};
use bytes::{Bytes, BytesMut};
use failure::Error;

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequestPacket {
    pub auth_plugin_name: Bytes,
    pub auth_plugin_data: Bytes,
}

impl Serialize for AuthenticationSwitchRequestPacket {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, 0xFE);
        encode_string_null(buf, &self.auth_plugin_name);
        encode_byte_eof(buf, &self.auth_plugin_data);

        Ok(())
    }
}
