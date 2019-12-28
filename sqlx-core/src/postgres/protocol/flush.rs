use crate::io::BufMut;
use crate::postgres::protocol::Encode;
use byteorder::NetworkEndian;

pub struct Flush;

impl Encode for Flush {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'H');
        buf.put_i32::<NetworkEndian>(4);
    }
}
