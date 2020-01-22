#![allow(dead_code)]
use crate::io::BufMut;
use crate::postgres::protocol::Encode;
use byteorder::NetworkEndian;

pub enum Close<'a> {
    Statement(&'a str),
    Portal(&'a str),
}

impl Encode for Close<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'C');

        let (kind, name) = match self {
            Close::Statement(name) => (b'S', name),
            Close::Portal(name) => (b'P', name),
        };

        // len + kind + nul + len(string)
        buf.put_i32::<NetworkEndian>((4 + 1 + 1 + name.len()) as i32);

        buf.push(kind);
        buf.put_str_nul(name);
    }
}

#[cfg(test)]
mod test {
    use super::{Close, Encode};

    #[test]
    fn it_encodes_close_portal() {
        let mut buf = Vec::new();
        let m = Close::Portal("__sqlx_p_1");

        m.encode(&mut buf);

        assert_eq!(buf, b"C\0\0\0\x10P__sqlx_p_1\0");
    }

    #[test]
    fn it_encodes_close_statement() {
        let mut buf = Vec::new();
        let m = Close::Statement("__sqlx_s_1");

        m.encode(&mut buf);

        assert_eq!(buf, b"C\0\0\0\x10S__sqlx_s_1\0");
    }
}
