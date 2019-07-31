use crate::mariadb::{Connection, Serialize};
use failure::Error;

pub struct ComResetConnection();

impl Serialize for ComResetConnection {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComResetConnection.into());

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_reset_conn() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComResetConnection().serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x01\0\0\x00\x1F");

        Ok(())
    }
}
