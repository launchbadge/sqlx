use super::{BufMut, Encode};

pub struct Flush;

impl Encode for Flush {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'H');
        buf.put_int_32(4);
    }
}
