#[derive(Debug)]
pub struct ComStmtReset {
    pub stmt_id: i32
}

impl crate::mariadb::Serialize for ComStmtReset {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::BinaryProtocol::ComStmtReset.into());
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
    fn it_encodes_com_stmt_reset() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComStmtReset {
            stmt_id: 1
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x05\0\0\x00\x1A\x01\0\0\0");

        Ok(())
    }
}
