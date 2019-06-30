use crate::Encode;
use bytes::BufMut;
use std::io;

#[derive(Debug)]
pub struct Terminate;

impl Encode for Terminate {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.reserve(5);
        buf.put_u8(b'X');
        buf.put_u32_be(4);

        Ok(())
    }
}
