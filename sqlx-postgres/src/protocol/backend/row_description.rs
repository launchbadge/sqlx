use bytes::{Buf, Bytes};
use bytestring::ByteString;
use sqlx_core::io::{BufExt, Deserialize};
use sqlx_core::Result;
use std::num::{NonZeroI16, NonZeroI32};

#[derive(Debug)]
pub(crate) struct RowDescription {
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug)]
pub(crate) struct Field {
    /// The name of the field.
    pub(crate) name: ByteString,

    /// If the field can be identified as a column of a specific table, the
    /// object ID of the table; otherwise zero.
    pub(crate) relation_id: Option<NonZeroI32>,

    /// If the field can be identified as a column of a specific table, the attribute number of
    /// the column; otherwise zero.
    pub(crate) relation_attribute_no: Option<NonZeroI16>,

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

impl Deserialize<'_> for RowDescription {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let cnt = buf.get_u16() as usize;
        let mut fields = Vec::with_capacity(cnt);

        for _ in 0..cnt {
            let name = buf.get_str_nul()?;
            let relation_id = buf.get_i32();
            let relation_attribute_no = buf.get_i16();
            let data_type_id = buf.get_u32();
            let data_type_size = buf.get_i16();
            let type_modifier = buf.get_i32();
            let format = buf.get_i16();

            fields.push(Field {
                name,
                relation_id: NonZeroI32::new(relation_id),
                relation_attribute_no: NonZeroI16::new(relation_attribute_no),
                data_type_id,
                data_type_size,
                type_modifier,
                format,
            })
        }

        Ok(Self { fields })
    }
}
