use super::Encode;
use crate::io::BufMut;
use byteorder::NetworkEndian;

pub struct CopyFail<'a> {
    pub error: &'a str,
}

impl Encode for CopyFail<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'f');
        // len + nul + len(string)
        buf.put_i32::<NetworkEndian>((4 + 1 + self.error.len()) as i32);
        buf.put_str_nul(&self.error);
    }
}
