use super::Decode;
use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use std::mem::size_of;

type ObjectId = u32;

#[derive(Debug)]
pub struct ParameterDescription {
    ids: Box<[ObjectId]>,
}

impl Decode for ParameterDescription {
    fn decode(src: &[u8]) -> Self {
        let count = BigEndian::read_u16(&*src) as usize;

        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * size_of::<u32>() + size_of::<u16>();
            ids.push(BigEndian::read_u32(&src[offset..]));
        }

        ParameterDescription { ids: ids.into_boxed_slice() }
    }
}

#[cfg(test)]
mod test {
    use super::{Decode, ParameterDescription};
    use bytes::Bytes;
    use std::io;

    #[test]
    fn it_decodes_parameter_description() {
        let src = b"\x00\x02\x00\x00\x00\x00\x00\x00\x05\x00";
        let desc = ParameterDescription::decode(src);

        assert_eq!(desc.ids.len(), 2);
        assert_eq!(desc.ids[0], 0x0000_0000);
        assert_eq!(desc.ids[1], 0x0000_0500);
    }

    #[test]
    fn it_decodes_empty_parameter_description() {
        let src = b"\x00\x00";
        let desc = ParameterDescription::decode(src);

        assert_eq!(desc.ids.len(), 0);
    }
}
