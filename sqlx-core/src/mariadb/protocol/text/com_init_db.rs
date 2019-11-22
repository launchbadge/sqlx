use super::TextProtocol;
use crate::{
    io::BufMut,
    mariadb::protocol::{Capabilities, Encode},
};

pub struct ComInitDb<'a> {
    pub schema_name: &'a str,
}

impl Encode for ComInitDb<'_> {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_INIT_DB Header : int<1>
        buf.put_u8(TextProtocol::ComInitDb as u8);

        // schema name : string<NUL>
        buf.put_str_nul(self.schema_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_init_db() {
        let mut buf = Vec::new();

        ComInitDb {
            schema_name: "portal",
        }
        .encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x02portal\0");
    }
}
