use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use bytes::Bytes;
use failure::Error;

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequestPacket {
    pub auth_plugin_name: Bytes,
    pub auth_plugin_data: Bytes,
}

impl Encode for AuthenticationSwitchRequestPacket {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.put_int_u8(0xFE);
        buf.put_string_null(&self.auth_plugin_name);
        buf.put_byte_eof(&self.auth_plugin_data);

        Ok(())
    }
}
