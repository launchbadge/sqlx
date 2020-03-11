use crate::io::BufMut;
use crate::postgres::protocol::{StatementId, Write};
use byteorder::{ByteOrder, NetworkEndian};

pub enum Describe<'a> {
    Statement(StatementId),
    Portal(&'a str),
}

impl Write for Describe<'_> {
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'D');

        let pos = buf.len();
        buf.put_i32::<NetworkEndian>(0); // skip over len

        match self {
            Describe::Statement(id) => {
                buf.push(b'S');
                id.write(buf);
            }

            Describe::Portal(name) => {
                buf.push(b'P');
                buf.put_str_nul(name);
            }
        };

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        NetworkEndian::write_i32(&mut buf[pos..], len as i32);
    }
}

#[cfg(test)]
mod test {
    use super::{Describe, Write};
    use crate::postgres::protocol::StatementId;

    #[test]
    fn it_writes_describe_portal() {
        let mut buf = Vec::new();
        let m = Describe::Portal("__sqlx_p_1");

        m.write(&mut buf);

        assert_eq!(buf, b"D\0\0\0\x10P__sqlx_p_1\0");
    }

    #[test]
    fn it_writes_describe_statement() {
        let mut buf = Vec::new();
        let m = Describe::Statement(StatementId(1));

        m.write(&mut buf);

        assert_eq!(buf, b"D\x00\x00\x00\x18S__sqlx_statement_1\x00");
    }
}
