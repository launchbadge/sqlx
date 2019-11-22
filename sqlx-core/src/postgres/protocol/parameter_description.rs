use super::Decode;
use crate::io::Buf;
use byteorder::NetworkEndian;
use std::io;

#[derive(Debug)]
pub struct ParameterDescription {
    pub ids: Box<[u32]>,
}

impl Decode for ParameterDescription {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let cnt = buf.get_u16::<NetworkEndian>()? as usize;
        let mut ids = Vec::with_capacity(cnt);

        for _ in 0..cnt {
            ids.push(buf.get_u32::<NetworkEndian>()?);
        }

        Ok(Self {
            ids: ids.into_boxed_slice(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Decode, ParameterDescription};

    #[test]
    fn it_decodes_parameter_description() {
        let buf = b"\x00\x02\x00\x00\x00\x00\x00\x00\x05\x00";
        let desc = ParameterDescription::decode(buf).unwrap();

        assert_eq!(desc.ids.len(), 2);
        assert_eq!(desc.ids[0], 0x0000_0000);
        assert_eq!(desc.ids[1], 0x0000_0500);
    }

    #[test]
    fn it_decodes_empty_parameter_description() {
        let buf = b"\x00\x00";
        let desc = ParameterDescription::decode(buf).unwrap();

        assert_eq!(desc.ids.len(), 0);
    }
}
