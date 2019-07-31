use bytes::Bytes;

#[derive(Debug)]
pub struct ComStmtPrepare {
    statement: Bytes
}

impl crate::mariadb::Serialize for ComStmtPrepare {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::BinaryProtocol::ComStmtPrepare.into());
        encoder.encode_string_eof(&self.statement);

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder, Serialize};

    #[test]
    fn it_encodes_com_stmt_prepare() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComStmtPrepare {
            statement: Bytes::from_static(b"SELECT * FROM users WHERE username = ?")
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], Bytes::from_static(b"\x27\0\0\x00\x16SELECT * FROM users WHERE username = ?"));

        Ok(())
    }
}

