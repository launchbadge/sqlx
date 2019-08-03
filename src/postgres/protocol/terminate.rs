use super::{BufMut, Encode};

pub struct Terminate;

impl Encode for Terminate {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'X');
        buf.put_int_32(4);
    }
}
