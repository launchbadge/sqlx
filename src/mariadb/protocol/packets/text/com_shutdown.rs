use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use failure::Error;

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

impl Encode for ComShutdown {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::TextProtocol::ComShutdown as u8);
        buf.put_int_u8(self.option as u8);

        buf.put_length();

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

    #[test]
    fn it_encodes_com_shutdown() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComShutdown {
            option: ShutdownOptions::ShutdownDefault,
        }
        .encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x02\0\0\x00\x0A\x00");

        Ok(())
    }
}
