use byteorder::LittleEndian;

use crate::io::BufMut;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::{Capabilities, Encode};

// https://dev.mysql.com/doc/internals/en/com-ping.html
#[derive(Debug)]
pub struct ComPing;

impl Encode for ComPing {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_PING : int<1>
        buf.put_u8(0x0e);
    }
}
