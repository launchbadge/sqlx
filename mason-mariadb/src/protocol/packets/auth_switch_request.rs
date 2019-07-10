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
    fn serialize(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.encoder.encode_int_1(0xFE);
        conn.encoder.encode_string_null(&self.auth_plugin_name);
        conn.encoder.encode_byte_eof(&self.auth_plugin_data);

        Ok(())
    }
}
