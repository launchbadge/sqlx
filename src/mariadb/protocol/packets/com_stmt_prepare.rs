use bytes::Bytes;

#[derive(Debug)]
pub struct ComStmtPrepare {
    statement: Bytes
}

impl crate::mariadb::Serialize for ComStmtPrepare {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(crate::mariadb::BinaryProtocol::ComStmtPrepare.into());
        encoder.encode_string_eof(&self.statement);

        encoder.encode_length();

        Ok(())
    }
}
