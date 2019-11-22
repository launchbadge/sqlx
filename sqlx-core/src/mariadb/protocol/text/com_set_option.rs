use crate::{
    io::BufMut,
    mariadb::protocol::{text::TextProtocol, Capabilities, Encode},
};
use byteorder::LittleEndian;

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum SetOptionOptions {
    MySqlOptionMultiStatementsOn = 0x00,
    MySqlOptionMultiStatementsOff = 0x01,
}

/// Enables or disables server option.
#[derive(Debug)]
pub struct ComSetOption {
    pub option: SetOptionOptions,
}

impl Encode for ComSetOption {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_SET_OPTION : int<1>
        buf.put_u8(TextProtocol::ComSetOption as u8);

        // option : int<2>
        buf.put_u16::<LittleEndian>(self.option as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_set_option() {
        let mut buf = Vec::new();

        ComSetOption {
            option: SetOptionOptions::MySqlOptionMultiStatementsOff,
        }
        .encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x1B\x01\0");
    }
}
