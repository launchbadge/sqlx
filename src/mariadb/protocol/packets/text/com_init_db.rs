use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use bytes::Bytes;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Encode for ComInitDb {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::TextProtocol::ComInitDb as u8);
        buf.put_string_null(&self.schema_name);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_init_db() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComInitDb {
            schema_name: Bytes::from_static(b"portal"),
        }
        .encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x08\0\0\x00\x02portal\0");

        Ok(())
    }
}
