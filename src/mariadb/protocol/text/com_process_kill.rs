use super::TextProtocol;
use crate::{
    io::BufMut,
    mariadb::protocol::{Capabilities, Encode},
};
use byteorder::LittleEndian;

/// Forces the server to terminate a specified connection.
pub struct ComProcessKill {
    pub process_id: u32,
}

impl Encode for ComProcessKill {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_PROCESS_KILL : int<1>
        buf.put_u8(TextProtocol::ComProcessKill as u8);

        // process id : int<4>
        buf.put_u32::<LittleEndian>(self.process_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_process_kill() {
        let mut buf = Vec::new();

        ComProcessKill { process_id: 1 }.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x0C\x01\0\0\0");
    }
}
