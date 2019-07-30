use crate::mariadb::{TextProtocol, Serialize, Connection};
use failure::Error;

pub struct ComResetConnection();

impl Serialize for ComResetConnection {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComResetConnection.into());

        encoder.encode_length();

        Ok(())
    }
}
