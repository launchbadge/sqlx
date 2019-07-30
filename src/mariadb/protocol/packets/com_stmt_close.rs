use std::convert::TryInto;

#[derive(Debug)]
pub struct ComStmtClose {
    stmt_id: i32
}

impl crate::mariadb::Serialize for ComStmtClose {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(crate::mariadb::BinaryProtocol::ComStmtClose.into());
        encoder.encode_int_i32(self.stmt_id);

        encoder.encode_length();

        Ok(())
    }
}
