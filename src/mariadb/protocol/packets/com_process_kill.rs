use crate::mariadb::{TextProtocol, Serialize, Connection};
use failure::Error;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Serialize for ComProcessKill {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComProcessKill.into());
        encoder.encode_int_u32(self.process_id);

        encoder.encode_length();

        Ok(())
    }
}
