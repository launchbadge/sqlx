use std::io;
use std::io::Cursor;

use bytes::Buf;
use sqlx_core::io::Stream;
use sqlx_core::mock::MockStream;

pub(crate) trait MySqlMockStreamExt {
    fn write_packet<'x>(&'x mut self, seq: u8, packet: &'x [u8]) -> io::Result<()>;

    fn read_packet(&mut self) -> io::Result<Vec<u8>>;

    fn expect_packet(&mut self, expected: &'static [u8]) -> io::Result<()>;
}

impl MySqlMockStreamExt for MockStream {
    fn write_packet<'x>(&'x mut self, seq: u8, packet: &'x [u8]) -> io::Result<()> {
        self.write(&packet.len().to_le_bytes()[..3])?;
        self.write(&[seq])?;
        self.write(packet)?;

        Ok(())
    }

    fn read_packet(&mut self) -> io::Result<Vec<u8>> {
        let mut packet = Vec::new();

        let mut header = [0_u8; 4];
        self.read(&mut header)?;

        packet.extend_from_slice(&header);

        let mut header_r = Cursor::new(header);
        let len = header_r.get_int_le(3) as usize;

        packet.resize(len + 4, 0);

        self.read(&mut packet[4..])?;

        Ok(packet)
    }

    fn expect_packet(&mut self, expected: &'static [u8]) -> io::Result<()> {
        let packet = self.read_packet()?;

        assert_eq!(expected, packet);

        Ok(())
    }
}
