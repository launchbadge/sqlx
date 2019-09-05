use crate::{io::BufMut, mariadb::Encode};

pub struct ComInitDb<'a> {
    pub schema_name: &'a str,
}

impl<'a> Encode for ComInitDb<'a> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComInitDb as u8);
        buf.put_str_nul(self.schema_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_encodes_com_init_db() -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComInitDb {
            schema_name: "portal",
        }
        .encode(&mut buf);

        assert_eq!(&buf[..], b"\x08\0\0\x00\x02portal\0");

        Ok(())
    }
}
