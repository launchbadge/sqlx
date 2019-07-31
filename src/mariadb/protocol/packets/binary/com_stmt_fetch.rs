#[derive(Debug)]
pub struct ComStmtFetch {
    pub stmt_id: i32,
    pub rows: u32,
}

impl crate::mariadb::Serialize for ComStmtFetch {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::BinaryProtocol::ComStmtFetch.into());
        encoder.encode_int_i32(self.stmt_id);
        encoder.encode_int_u32(self.rows);

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder, Serialize};

    #[test]
    fn it_encodes_com_stmt_fetch() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComStmtFetch {
            stmt_id: 1,
            rows: 10,
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x09\0\0\x00\x1C\x01\0\0\0\x0A\0\0\0");

        Ok(())
    }
}
