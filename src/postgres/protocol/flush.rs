use super::Encode;
use crate::io::BufMut;
use byteorder::NetworkEndian;

pub struct Flush;

impl Encode for Flush {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'H');
        buf.put_i32::<NetworkEndian>(4);
    }
}
