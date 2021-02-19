use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use super::Capabilities;

/// Response from a successful `COM_STMT_PREPARE`.
///
/// <https://dev.mysql.com/doc/internals/en/com-stmt-prepare-response.html#packet-COM_STMT_PREPARE_OK>
/// <https://mariadb.com/kb/en/com_stmt_prepare/#com_stmt_prepare_ok>
///
#[derive(Debug)]
pub(crate) struct PrepareOk {
    pub(crate) statement_id: u32,
    pub(crate) columns: u16,
    pub(crate) params: u16,
    pub(crate) warnings: u16,
}

impl Deserialize<'_, Capabilities> for PrepareOk {
    fn deserialize_with(mut buf: Bytes, _: Capabilities) -> Result<Self> {
        let status = buf.get_u8();
        debug_assert!(status == 0x00);

        let statement_id = buf.get_u32_le();
        let columns = buf.get_u16_le();
        let params = buf.get_u16_le();
        let warnings = buf.get_u16_le();

        Ok(Self { statement_id, columns, params, warnings })
    }
}
