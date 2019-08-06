use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Default, Clone, Copy)]
pub struct PacketHeader {
    pub length: u32,
    pub seq_no: u8,
}

impl PacketHeader {
    pub fn size() -> usize {
        4
    }

    pub fn combined_length(&self) -> usize {
        PacketHeader::size() + self.length as usize
    }
}

impl core::convert::TryFrom<&[u8]> for PacketHeader {
    type Error = failure::Error;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 4 {
            failure::bail!("Buffer length is too short")
        } else {
            let packet = PacketHeader {
                length: LittleEndian::read_u24(&buffer),
                seq_no: buffer[3],
            };
            if packet.length == 0 && packet.seq_no == 0 {
                failure::bail!("Length and seq_no cannot be zero");
            }
            Ok(packet)
        }
    }
}
