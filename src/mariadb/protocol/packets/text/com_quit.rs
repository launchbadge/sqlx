use crate::mariadb::{Connection, Serialize};
use failure::Error;

pub struct ComQuit();

impl Serialize for ComQuit {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComQuit.into());

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_quit() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComQuit().serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x01\0\0\x00\x01");

        Ok(())
    }
}

