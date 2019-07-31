use crate::mariadb::{Connection, Serialize};
use failure::Error;

#[derive(Clone, Copy)]
pub enum SetOptionOptions {
    MySqlOptionMultiStatementsOn = 0x00,
    MySqlOptionMultiStatementsOff = 0x01,
}

pub struct ComSetOption {
    pub option: SetOptionOptions,
}

impl Serialize for ComSetOption {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComSetOption.into());
        encoder.encode_int_u16(self.option.into());

        encoder.encode_length();

        Ok(())
    }
}

// Helper method to easily transform into u16
impl Into<u16> for SetOptionOptions {
    fn into(self) -> u16 {
        self as u16
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_set_option() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComSetOption {
            option: SetOptionOptions::MySqlOptionMultiStatementsOff
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x03\0\0\x00\x1B\x01\0");

        Ok(())
    }
}

