use super::{BufMut, Encode};

pub struct CopyFail<'a> {
    pub error: &'a str,
}

impl Encode for CopyFail<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'f');
        // len + nul + len(string)
        buf.put_int_32((4 + 1 + self.error.len()) as i32);
        buf.put_str(&self.error);
    }
}
