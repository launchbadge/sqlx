use super::Encode;
use std::io;

#[derive(Debug)]
pub enum DescribeKind {
    Portal,
    PreparedStatement,
}

#[derive(Debug)]
pub struct Describe<'a> {
    kind: DescribeKind,
    name: &'a str,
}

impl<'a> Describe<'a> {
    pub fn new(kind: DescribeKind, name: &'a str) -> Self {
        Self { kind, name }
    }
}

impl<'a> Encode for Describe<'a> {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'D');

        let len = 4 + self.name.len() + 1 + 4;
        buf.extend_from_slice(&(len as i32).to_be_bytes());

        match &self.kind {
            DescribeKind::Portal => buf.push(b'P'),
            DescribeKind::PreparedStatement => buf.push(b'S'),
        };

        buf.extend_from_slice(self.name.as_bytes());
        buf.push(b'\0');

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Describe, DescribeKind, Encode};
    use std::io;

    #[test]
    fn it_encodes_describe_portal() -> io::Result<()> {
        let mut buf = vec![];
        Describe::new(DescribeKind::Portal, "ABC123").encode(&mut buf)?;
        assert_eq!(&buf, b"D\x00\x00\x00\x0fPABC123\x00");

        Ok(())
    }

    #[test]
    fn it_encodes_describe_statement() -> io::Result<()> {
        let mut buf = vec![];
        Describe::new(DescribeKind::PreparedStatement, "95 apples").encode(&mut buf)?;
        assert_eq!(&buf, b"D\x00\x00\x00\x12S95 apples\x00");

        Ok(())
    }
}
