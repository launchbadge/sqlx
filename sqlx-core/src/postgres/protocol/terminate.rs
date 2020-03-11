use crate::io::BufMut;
use crate::postgres::protocol::Write;
use byteorder::NetworkEndian;

pub struct Terminate;

impl Write for Terminate {
    #[inline]
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'X');
        buf.put_i32::<NetworkEndian>(4);
    }
}
