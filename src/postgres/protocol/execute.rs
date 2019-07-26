use super::Encode;
use std::io;

#[derive(Debug)]
pub struct Execute<'a> {
    portal: &'a str,
    limit: i32,
}

impl<'a> Execute<'a> {
    pub fn new(portal: &'a str, limit: i32) -> Self {
        Self { portal, limit }
    }
}

impl<'a> Encode for Execute<'a> {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'E');

        let len = 4 + self.portal.len() + 1 + 4;
        buf.extend_from_slice(&(len as i32).to_be_bytes());

        // portal
        buf.extend_from_slice(self.portal.as_bytes());
        buf.push(b'\0');

        // limit
        buf.extend_from_slice(&self.limit.to_be_bytes());

        Ok(())
    }
}
