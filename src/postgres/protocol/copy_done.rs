use super::{Encode};
use crate::io::BufMut;
use byteorder::NetworkEndian;

// TODO: Implement Decode

pub struct CopyDone;

impl Encode for CopyDone {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'c');
        buf.put_i32::<NetworkEndian>(4);
    }
}
