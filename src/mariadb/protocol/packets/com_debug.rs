use crate::mariadb::{Connection, TextProtocol, Serialize};
use failure::Error;

pub struct ComDebug();

impl Serialize for ComDebug {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComDebug.into());

        encoder.encode_length();

        Ok(())
    }
}
