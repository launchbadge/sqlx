use super::Encode;
use std::io;

#[derive(Debug)]
pub struct Sync;

impl Encode for Sync {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'S');
        buf.extend_from_slice(&4_i32.to_be_bytes());

        Ok(())
    }
}
