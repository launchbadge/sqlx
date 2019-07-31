use std::convert::TryInto;

#[derive(Debug)]
pub struct ComStmtClose {
    stmt_id: i32
}

impl crate::mariadb::Serialize for ComStmtClose {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::BinaryProtocol::ComStmtClose.into());
        encoder.encode_int_i32(self.stmt_id);

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder, Serialize};

    #[test]
    fn it_encodes_com_stmt_close() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComStmtClose {
            stmt_id: 1
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x05\0\0\x00\x19\x01\0\0\0");

        Ok(())
    }
}
