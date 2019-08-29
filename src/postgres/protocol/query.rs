use super::Encode;
use crate::io::BufMut;
use byteorder::NetworkEndian;

pub struct Query<'a>(pub &'a str);

impl Encode for Query<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'Q');

        // len + query + nul
        buf.put_i32::<NetworkEndian>((4 + self.0.len() + 1) as i32);

        buf.put_str_nul(self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{BufMut, Encode, Query};

    const QUERY_SELECT_1: &[u8] = b"Q\0\0\0\rSELECT 1\0";

    #[test]
    fn it_encodes_query() {
        let mut buf = Vec::new();
        let m = Query("SELECT 1");

        m.encode(&mut buf);

        assert_eq!(buf, QUERY_SELECT_1);
    }
}
