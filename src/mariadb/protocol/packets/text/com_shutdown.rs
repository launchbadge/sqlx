use crate::mariadb::{Connection, Serialize};
use failure::Error;

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

impl Serialize for ComShutdown {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComShutdown.into());
        encoder.encode_int_u8(self.option.into());

        encoder.encode_length();

        Ok(())
    }
}

// Helper method to easily transform into u8
impl Into<u8> for ShutdownOptions {
    fn into(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_shutdown() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComShutdown {
            option: ShutdownOptions::ShutdownDefault
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x02\0\0\x00\x0A\x00");

        Ok(())
    }
}

