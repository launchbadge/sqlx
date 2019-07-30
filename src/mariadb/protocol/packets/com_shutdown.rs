use crate::mariadb::{TextProtocol, Serialize, Connection};
use failure::Error;

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

impl Serialize for ComShutdown {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComShutdown.into());
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
