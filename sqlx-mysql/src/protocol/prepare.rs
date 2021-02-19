use sqlx_core::io::Serialize;
use sqlx_core::Result;

use super::Command;

/// Creates a prepared statement from the passed query string.
///
/// <https://dev.mysql.com/doc/internals/en/com-stmt-prepare.html>
/// <https://mariadb.com/kb/en/com_stmt_prepare/>
///
#[derive(Debug)]
pub(crate) struct Prepare<'q> {
    pub(crate) sql: &'q str,
}

impl Serialize<'_> for Prepare<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(0x16);
        buf.extend_from_slice(self.sql.as_bytes());

        Ok(())
    }
}

impl Command for Prepare<'_> {}
