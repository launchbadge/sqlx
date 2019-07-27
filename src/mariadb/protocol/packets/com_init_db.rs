use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
use bytes::Bytes;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Serialize for ComInitDb {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComInitDb.into());
        encoder.encode_string_null(&self.schema_name);

        Ok(())
    }
}
