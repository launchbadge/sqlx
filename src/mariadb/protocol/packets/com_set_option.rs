use crate::mariadb::{TextProtocol, Serialize, Connection};
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
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComSetOption.into());
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
