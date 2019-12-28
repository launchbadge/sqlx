use crate::io::BufMut;
use crate::postgres::protocol::Encode;
use byteorder::NetworkEndian;

pub struct Terminate;

impl Encode for Terminate {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'X');
        buf.put_i32::<NetworkEndian>(4);
    }
}
