use crate::io::BufMut;
use crate::postgres::protocol::Write;
use byteorder::NetworkEndian;

pub struct Query<'a>(pub &'a str);

impl Write for Query<'_> {
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'Q');

        // len + query + nul
        buf.put_i32::<NetworkEndian>((4 + self.0.len() + 1) as i32);

        buf.put_str_nul(self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{Query, Write};

    const QUERY_SELECT_1: &[u8] = b"Q\0\0\0\rSELECT 1\0";

    #[test]
    fn it_writes_query() {
        let mut buf = Vec::new();
        let m = Query("SELECT 1");

        m.write(&mut buf);

        assert_eq!(buf, QUERY_SELECT_1);
    }
}
