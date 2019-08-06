use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use failure::Error;

pub struct ComStatistics();

impl Encode for ComStatistics {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::TextProtocol::ComStatistics.into());

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_statistics() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComStatistics().encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x01\0\0\x00\x09");

        Ok(())
    }
}
