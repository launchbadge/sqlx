use super::super::serialize::Serialize;
use crate::connection::Connection;
use bytes::Bytes;
use failure::Error;

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequestPacket {
    pub auth_plugin_name: Bytes,
    pub auth_plugin_data: Bytes,
}

impl Serialize for AuthenticationSwitchRequestPacket {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::connection::ConnContext, encoder: &mut crate::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(0xFE);
        encoder.encode_string_null(&self.auth_plugin_name);
        encoder.encode_byte_eof(&self.auth_plugin_data);

        Ok(())
    }
}
