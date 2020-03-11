use crate::io::BufMut;
use crate::postgres::protocol::Write;
use byteorder::NetworkEndian;

pub struct Sync;

impl Write for Sync {
    #[inline]
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'S');
        buf.put_i32::<NetworkEndian>(4);
    }
}
