use super::{BufMut, Encode};

pub struct Query<'a>(pub &'a str);

impl Encode for Query<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'Q');

        // len + query + nul
        buf.put_int_32((4 + self.0.len() + 1) as i32);

        buf.put_str(self.0);
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
