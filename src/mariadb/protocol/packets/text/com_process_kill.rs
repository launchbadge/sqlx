use crate::mariadb::{Connection, Serialize};
use failure::Error;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Serialize for ComProcessKill {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComProcessKill.into());
        encoder.encode_int_u32(self.process_id);

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_process_kill() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComProcessKill {
            process_id: 1,
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x05\0\0\x00\x0C\x01\0\0\0");

        Ok(())
    }
}
