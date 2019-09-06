use crate::{io::BufMut, mariadb::Encode};
use byteorder::LittleEndian;

#[derive(Clone, Copy)]
pub enum SetOptionOptions {
    MySqlOptionMultiStatementsOn = 0x00,
    MySqlOptionMultiStatementsOff = 0x01,
}

pub struct ComSetOption {
    pub option: SetOptionOptions,
}

impl Encode for ComSetOption {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComSetOption as u8);
        buf.put_u16::<LittleEndian>(self.option.into());
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

    #[test]
    fn it_encodes_com_set_option() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComSetOption {
            option: SetOptionOptions::MySqlOptionMultiStatementsOff,
        }
        .encode(&mut buf);

        assert_eq!(&buf[..], b"\x03\0\0\x00\x1B\x01\0");

        Ok(())
    }
}
