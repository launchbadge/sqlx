use crate::{
    io::BufMut,
    mariadb::{
        io::BufMutExt,
        protocol::{Capabilities, Encode},
    },
};

/// Sends the server an SQL statement to be executed immediately.
pub struct ComQuery<'a> {
    pub sql_statement: &'a str,
}

impl<'a> Encode for ComQuery<'a> {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        buf.put_u8(super::TextProtocol::ComQuery as u8);
        buf.put_str(&self.sql_statement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_query() {
        let mut buf = Vec::new();

        ComQuery {
            sql_statement: "SELECT * FROM users",
        }
        .encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf, b"\x03SELECT * FROM users");
    }
}
