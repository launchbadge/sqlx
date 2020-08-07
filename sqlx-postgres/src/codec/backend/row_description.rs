use crate::io::get_str_bytes;
use bytes::{Buf, Bytes};
use smallvec::SmallVec;
use sqlx_core::{error::Error, io::Decode};
use std::str::from_utf8;

#[derive(Debug)]
pub(crate) struct RowDescription {
    pub(crate) fields: SmallVec<[Field; 6]>,
}

#[derive(Debug)]
pub(crate) struct Field {
    /// The name of the field.
    pub(crate) name: Bytes,

    /// If the field can be identified as a column of a specific table, the
    /// object ID of the table; otherwise zero.
    pub(crate) relation_id: Option<i32>,

    /// If the field can be identified as a column of a specific table, the attribute number of
    /// the column; otherwise zero.
    pub(crate) relation_attribute_no: Option<i16>,

    /// The object ID of the field's data type.
    pub(crate) data_type_id: u32,

    /// The data type size (see pg_type.typlen). Note that negative values denote
    /// variable-width types.
    pub(crate) data_type_size: i16,

    /// The type modifier (see pg_attribute.atttypmod). The meaning of the
    /// modifier is type-specific.
    pub(crate) type_modifier: i32,

    /// The format code being used for the field.
    pub(crate) format: i16,
}

impl Field {
    pub(crate) fn name(&self) -> Result<&str, Error> {
        from_utf8(self.name.as_ref()).map_err(Error::protocol)
    }
}

impl Decode<'_> for RowDescription {
    fn decode_with(mut buf: Bytes, _: ()) -> Result<Self, Error> {
        let cnt = buf.get_u16();
        let mut fields = SmallVec::with_capacity(cnt as usize);

        for _ in 0..cnt {
            let name = get_str_bytes(&mut buf)?;
            let relation_id = buf.get_i32();
            let relation_attribute_no = buf.get_i16();
            let data_type_id = buf.get_u32();
            let data_type_size = buf.get_i16();
            let type_modifier = buf.get_i32();
            let format = buf.get_i16();

            fields.push(Field {
                name,
                relation_id: if relation_id == 0 {
                    None
                } else {
                    Some(relation_id)
                },
                relation_attribute_no: if relation_attribute_no == 0 {
                    None
                } else {
                    Some(relation_attribute_no)
                },
                data_type_id,
                data_type_size,
                type_modifier,
                format,
            })
        }

        Ok(Self { fields })
    }
}
