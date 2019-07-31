use crate::mariadb::{Connection, Serialize};
use bytes::Bytes;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Serialize for ComInitDb {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComInitDb.into());
        encoder.encode_string_null(&self.schema_name);

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_init_db() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComInitDb {
          schema_name: Bytes::from_static(b"portal"),
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x08\0\0\x00\x02portal\0");

        Ok(())
    }
}

