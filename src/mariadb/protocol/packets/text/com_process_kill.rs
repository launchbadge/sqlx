use crate::{io::BufMut, mariadb::Encode};
use byteorder::LittleEndian;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Encode for ComProcessKill {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComProcessKill.into());
        buf.put_u32::<LittleEndian>(self.process_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_encodes_com_process_kill() -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComProcessKill { process_id: 1 }.encode(&mut buf);

        assert_eq!(&buf[..], b"\x05\0\0\x00\x0C\x01\0\0\0");

        Ok(())
    }
}
