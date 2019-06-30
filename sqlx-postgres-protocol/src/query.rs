use crate::Encode;
use bytes::BufMut;
use std::io;

#[derive(Debug)]
pub struct Query<'a>(&'a str);

impl<'a> Query<'a> {
    pub fn new(query: &'a str) -> Self {
        Self(query)
    }
}

impl Encode for Query<'_> {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let len = self.0.len() + 4 + 1;
        buf.reserve(len + 1);
        buf.put_u8(b'Q');
        buf.put_u32_be(len as u32);
        buf.put(self.0);
        buf.put_u8(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Query;
    use crate::Encode;
    use std::io;

    const QUERY_SELECT_1: &[u8] = b"Q\0\0\0\rSELECT 1\0";

    #[test]
    fn it_encodes_query() -> io::Result<()> {
        let message = Query::new("SELECT 1");

        let mut buf = Vec::new();
        message.encode(&mut buf)?;

        assert_eq!(&*buf, QUERY_SELECT_1);

        Ok(())
    }
}
