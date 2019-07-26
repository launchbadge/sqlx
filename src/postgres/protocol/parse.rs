use super::Encode;
use std::io;

#[derive(Debug)]
pub struct Parse<'a> {
    portal: &'a str,
    query: &'a str,
    param_types: &'a [i32],
}

impl<'a> Parse<'a> {
    pub fn new(portal: &'a str, query: &'a str, param_types: &'a [i32]) -> Self {
        Self {
            portal,
            query,
            param_types,
        }
    }
}

impl<'a> Encode for Parse<'a> {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'P');

        let len = 4 + self.portal.len() + 1 + self.query.len() + 1 + 2 + self.param_types.len() * 4;

        buf.extend_from_slice(&(len as i32).to_be_bytes());

        buf.extend_from_slice(self.portal.as_bytes());
        buf.push(b'\0');

        buf.extend_from_slice(self.query.as_bytes());
        buf.push(b'\0');

        buf.extend_from_slice(&(self.param_types.len() as i16).to_be_bytes());

        for param_type in self.param_types {
            buf.extend_from_slice(&param_type.to_be_bytes());
        }

        Ok(())
    }
}
