use crate::io::BufMut;
use crate::mysql::protocol::Capabilities;

pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities);
}

impl Encode for &'_ [u8] {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        buf.put_bytes(self);
    }
}
