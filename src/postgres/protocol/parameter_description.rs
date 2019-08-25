use super::{Buf, Decode};
use byteorder::{BigEndian, ByteOrder};
use std::{io, mem::size_of};

type ObjectId = u32;

#[derive(Debug)]
pub struct ParameterDescription {
    ids: Box<[ObjectId]>,
}

impl Decode for ParameterDescription {
    fn decode(mut src: &[u8]) -> io::Result<Self> {
        let count = src.get_u16()?;
        let mut ids = Vec::with_capacity(count as usize);

        for i in 0..count {
            ids.push(src.get_u32()?);
        }

        Ok(ParameterDescription {
            ids: ids.into_boxed_slice(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Decode, ParameterDescription};
    use std::io;

    #[test]
    fn it_decodes_parameter_description() {
        let src = b"\x00\x02\x00\x00\x00\x00\x00\x00\x05\x00";
        let desc = ParameterDescription::decode(src).unwrap();

        assert_eq!(desc.ids.len(), 2);
        assert_eq!(desc.ids[0], 0x0000_0000);
        assert_eq!(desc.ids[1], 0x0000_0500);
    }

    #[test]
    fn it_decodes_empty_parameter_description() {
        let src = b"\x00\x00";
        let desc = ParameterDescription::decode(src).unwrap();

        assert_eq!(desc.ids.len(), 0);
    }
}
