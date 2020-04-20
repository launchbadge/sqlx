use crate::io::Encode;

pub struct Terminate;

impl Encode for Terminate {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'X');
        buf.extend(&4_u32.to_be_bytes());
    }
}
