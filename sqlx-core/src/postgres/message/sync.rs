use crate::io::Encode;

#[derive(Debug)]
pub struct Sync;

impl Encode for Sync {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'S');
        buf.extend(&4_i32.to_be_bytes());
    }
}
