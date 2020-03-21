use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::io::BufExt;
use crate::mysql::protocol::{FieldFlags, TypeId};
use crate::mysql::MySql;

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_com_query_response_text_resultset_column_definition.html
// https://mariadb.com/kb/en/resultset/#column-definition-packet
#[derive(Debug)]
pub struct ColumnDefinition {
    pub schema: Option<Box<str>>,

    pub table_alias: Option<Box<str>>,
    pub table: Option<Box<str>>,

    pub column_alias: Option<Box<str>>,
    pub column: Option<Box<str>>,

    pub char_set: u16,

    pub max_size: u32,

    pub type_id: TypeId,

    pub flags: FieldFlags,

    pub decimals: u8,
}

impl ColumnDefinition {
    pub fn name(&self) -> Option<&str> {
        self.column_alias.as_deref().or(self.column.as_deref())
    }
}

impl ColumnDefinition {
    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<MySql, Self> {
        // catalog : string<lenenc>
        let catalog = buf.get_str_lenenc::<LittleEndian>()?;

        if catalog != Some("def") {
            return Err(protocol_err!(
                "expected ColumnDefinition (\"def\"); received {:?}",
                catalog
            ))?;
        }

        let schema = buf.get_str_lenenc::<LittleEndian>()?.map(Into::into);
        let table_alias = buf.get_str_lenenc::<LittleEndian>()?.map(Into::into);
        let table = buf.get_str_lenenc::<LittleEndian>()?.map(Into::into);
        let column_alias = buf.get_str_lenenc::<LittleEndian>()?.map(Into::into);
        let column = buf.get_str_lenenc::<LittleEndian>()?.map(Into::into);

        let len_fixed_fields = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0);

        if len_fixed_fields != 0x0c {
            return Err(protocol_err!(
                "expected ColumnDefinition (0x0c); received {:?}",
                len_fixed_fields
            ))?;
        }

        let char_set = buf.get_u16::<LittleEndian>()?;
        let max_size = buf.get_u32::<LittleEndian>()?;

        let type_id = buf.get_u8()?;
        let flags = buf.get_u16::<LittleEndian>()?;
        let decimals = buf.get_u8()?;

        Ok(Self {
            schema,
            table,
            table_alias,
            column,
            column_alias,
            char_set,
            max_size,
            type_id: TypeId(type_id),
            flags: FieldFlags::from_bits_truncate(flags),
            decimals,
        })
    }
}
