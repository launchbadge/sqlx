use crate::mariadb::{TextProtocol, Serialize, Connection};
use bytes::Bytes;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Serialize for ComInitDb {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComInitDb.into());
        encoder.encode_string_null(&self.schema_name);

        encoder.encode_length();

        Ok(())
    }
}
