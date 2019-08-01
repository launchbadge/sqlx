use byteorder::LittleEndian;
use byteorder::ByteOrder;

#[derive(Debug, Default)]
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
            Ok(PacketHeader {
                length: LittleEndian::read_u24(&buffer),
                seq_no: buffer[3],
            })
        }
    }
}
