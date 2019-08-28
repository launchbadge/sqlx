use super::{Encode};
use crate::io::BufMut;
use byteorder::NetworkEndian;

pub struct Terminate;

impl Encode for Terminate {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'X');
        buf.put_i32::<NetworkEndian>(4);
    }
}
