use super::TextProtocol;
use crate::{
    io::BufMut,
    mariadb::protocol::{Capabilities, Encode},
};

/// Resets a connection without re-authentication.
#[derive(Debug)]
pub struct ComResetConnection;

impl Encode for ComResetConnection {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_RESET_CONNECTION Header : int<1>
        buf.put_u8(TextProtocol::ComResetConnection as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_reset_conn() {
        let mut buf = Vec::new();

        ComResetConnection.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x1F");
    }
}
