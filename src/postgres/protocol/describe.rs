use super::Encode;
use crate::io::BufMut;
use byteorder::NetworkEndian;

#[repr(u8)]
pub enum DescribeKind {
    PreparedStatement,
    Portal,
}

pub struct Describe<'a> {
    kind: DescribeKind,

    /// The name of the prepared statement or portal to describe (an empty string selects the
    /// unnamed prepared statement or portal).
    name: &'a str,
}

impl Encode for Describe<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'D');
        // len + kind + nul + len(string)
        buf.put_i32::<NetworkEndian>((4 + 1 + 1 + self.name.len()) as i32);
        buf.push(match self.kind {
            DescribeKind::PreparedStatement => b'S',
            DescribeKind::Portal => b'P',
        });
        buf.put_str_nul(self.name);
    }
}

#[cfg(test)]
mod test {
    use super::{BufMut, Describe, DescribeKind, Encode};

    #[test]
    fn it_encodes_describe_portal() {
        let mut buf = Vec::new();
        let m = Describe {
            kind: DescribeKind::Portal,
            name: "__sqlx_p_1",
        };

        m.encode(&mut buf);

        assert_eq!(buf, b"D\0\0\0\x10P__sqlx_p_1\0");
    }

    #[test]
    fn it_encodes_describe_statement() {
        let mut buf = Vec::new();
        let m = Describe {
            kind: DescribeKind::PreparedStatement,
            name: "__sqlx_s_1",
        };

        m.encode(&mut buf);

        assert_eq!(buf, b"D\0\0\0\x10S__sqlx_s_1\0");
    }
}
