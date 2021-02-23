use bytes::{Buf, Bytes};
use bytestring::ByteString;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use super::ColumnFlags;
use crate::io::MySqlBufExt;

/// Describes a column in the result set.
///
/// <https://mariadb.com/kb/en/result-set-packets/#column-definition-packet>
/// <https://dev.mysql.com/doc/internals/en/com-query-response.html#packet-Protocol::ColumnDefinition>
///
#[derive(Debug)]
pub(crate) struct ColumnDefinition {
    pub(crate) schema: ByteString,
    pub(crate) table_alias: ByteString,
    pub(crate) table: ByteString,
    pub(crate) alias: ByteString,
    pub(crate) name: ByteString,
    pub(crate) charset: u16,
    pub(crate) max_size: u32,
    pub(crate) ty: u8,
    pub(crate) flags: ColumnFlags,
    pub(crate) decimals: u8,
}

impl Deserialize<'_> for ColumnDefinition {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let catalog = buf.get_str_lenenc()?;

        // we are told that this always "def"
        debug_assert_eq!(catalog, "def");

        let schema = buf.get_str_lenenc()?;
        let table_alias = buf.get_str_lenenc()?;
        let table = buf.get_str_lenenc()?;
        let alias = buf.get_str_lenenc()?;
        let name = buf.get_str_lenenc()?;

        let fixed_len_fields_len = buf.get_uint_lenenc();

        // we are told that this is *always* 0x0c
        debug_assert_eq!(fixed_len_fields_len, 0x0c);

        let charset = buf.get_u16_le();
        let max_size = buf.get_u32_le();
        let ty = buf.get_u8();
        let flags = ColumnFlags::from_bits_truncate(buf.get_u16_le());
        let decimals = buf.get_u8();

        Ok(Self { schema, table_alias, table, alias, name, charset, max_size, ty, flags, decimals })
    }
}
