use crate::{
    io::BufMut,
    mariadb::protocol::{text::TextProtocol, Capabilities, Encode},
};

pub struct ComSleep;

impl Encode for ComSleep {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_SLEEP : int<1>
        buf.put_u8(TextProtocol::ComSleep as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_sleep() {
        let mut buf = Vec::new();

        ComSleep.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x00");
    }
}
