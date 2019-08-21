use super::{BufMut, Encode};

pub struct Sync;

impl Encode for Sync {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'S');
        buf.put_int_32(4);
    }
}
