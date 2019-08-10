use super::{BufMut, Encode};

// TODO: Implement Decode

pub struct CopyDone;

impl Encode for CopyDone {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'c');
        buf.put_int_32(4);
    }
}
