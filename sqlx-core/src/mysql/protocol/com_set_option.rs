use byteorder::LittleEndian;

use crate::io::BufMut;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::{Capabilities, Encode};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/mysql__com_8h.html#a53f60000da139fc7d547db96635a2c02
#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum SetOption {
    MultiStatementsOn = 0x00,
    MultiStatementsOff = 0x01,
}

// https://dev.mysql.com/doc/internals/en/com-set-option.html
#[derive(Debug)]
pub struct ComSetOption {
    pub option: SetOption,
}

impl Encode for ComSetOption {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_SET_OPTION : int<1>
        buf.put_u8(0x1a);

        // option : int<2>
        buf.put_u16::<LittleEndian>(self.option as u16);
    }
}
