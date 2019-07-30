use crate::mariadb::{TextProtocol, Serialize, Connection};
use failure::Error;

pub struct ComPing();

impl Serialize for ComPing {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComPing.into());

        encoder.encode_length();

        Ok(())
    }
}
