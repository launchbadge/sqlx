use super::Decode;
use crate::io::Buf;
use byteorder::NetworkEndian;
use std::io;
use std::io::BufRead;

#[derive(Debug)]
pub struct RowDescription {
    pub fields: Box<[RowField]>,
}

#[derive(Debug)]
pub struct RowField {
    pub name: String,
    pub table_id: u32,
    pub attr_num: i16,
    pub type_id: u32,
    pub type_size: i16,
    pub type_mod: i32,
    pub format_code: i16
}

impl Decode for RowDescription {
    fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let cnt = buf.get_u16::<NetworkEndian>()? as usize;
        let mut fields = Vec::with_capacity(cnt);

        for _ in 0..cnt {
            fields.push(dbg!(RowField {
                name: super::read_string(&mut buf)?,
                table_id: buf.get_u32::<NetworkEndian>()?,
                attr_num: buf.get_i16::<NetworkEndian>()?,
                type_id: buf.get_u32::<NetworkEndian>()?,
                type_size: buf.get_i16::<NetworkEndian>()?,
                type_mod: buf.get_i32::<NetworkEndian>()?,
                format_code: buf.get_i16::<NetworkEndian>()?,
            }));
        }

        Ok(Self {
            fields: fields.into_boxed_slice(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Decode, RowDescription};

    #[test]
    fn it_decodes_parameter_description() {
        let buf = b"\x00\x02\x00\x00\x00\x00\x00\x00\x05\x00";
        let desc = RowDescription::decode(buf).unwrap();

        assert_eq!(desc.fields.len(), 2);
        assert_eq!(desc.fields[0].type_id, 0x0000_0000);
        assert_eq!(desc.fields[1].type_id, 0x0000_0500);
    }

    #[test]
    fn it_decodes_empty_parameter_description() {
        let buf = b"\x00\x00";
        let desc = RowDescription::decode(buf).unwrap();

        assert_eq!(desc.fields.len(), 0);
    }
}
