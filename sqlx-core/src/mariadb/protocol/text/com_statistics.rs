use crate::{
    io::BufMut,
    mariadb::protocol::{text::TextProtocol, Capabilities, Encode},
};

#[derive(Debug)]
pub struct ComStatistics;

impl Encode for ComStatistics {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_STATISTICS : int<1>
        buf.put_u8(TextProtocol::ComStatistics as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_statistics() {
        let mut buf = Vec::new();

        ComStatistics.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x09");
    }
}
