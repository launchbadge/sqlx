use super::super::{client::TextProtocol, serialize::Serialize};
use crate::connection::Connection;
use bytes::Bytes;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Serialize for ComInitDb {
    fn serialize<'a, 'b>(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.encoder.encode_int_1(TextProtocol::ComInitDb.into());
        conn.encoder.encode_string_null(&self.schema_name);

        Ok(())
    }
}
