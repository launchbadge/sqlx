use super::{BufMut, Encode};

// TODO: Separate into two structs, ClosePortal and CloseStatement (?)

#[repr(u8)]
pub enum CloseKind {
    PreparedStatement,
    Portal,
}

pub struct Close<'a> {
    kind: CloseKind,

    /// The name of the prepared statement or portal to close (an empty string selects the
    /// unnamed prepared statement or portal).
    name: &'a str,
}

impl Encode for Close<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'C');
        // len + kind + nul + len(string)
        buf.put_int_32((4 + 1 + 1 + self.name.len()) as i32);
        buf.put_byte(match self.kind {
            CloseKind::PreparedStatement => b'S',
            CloseKind::Portal => b'P',
        });
        buf.put_str(self.name);
    }
}

#[cfg(test)]
mod test {
    use super::{BufMut, Close, CloseKind, Encode};

    #[test]
    fn it_encodes_close_portal() {
        let mut buf = Vec::new();
        let m = Close {
            kind: CloseKind::Portal,
            name: "__sqlx_p_1",
        };

        m.encode(&mut buf);

        assert_eq!(buf, b"C\0\0\0\x10P__sqlx_p_1\0");
    }

    #[test]
    fn it_encodes_close_statement() {
        let mut buf = Vec::new();
        let m = Close {
            kind: CloseKind::PreparedStatement,
            name: "__sqlx_s_1",
        };

        m.encode(&mut buf);

        assert_eq!(buf, b"C\0\0\0\x10S__sqlx_s_1\0");
    }
}
