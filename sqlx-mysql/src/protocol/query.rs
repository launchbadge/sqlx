use sqlx_core::io::Serialize;
use sqlx_core::Result;

use super::Command;

/// Send the server a text-based query that is executed immediately.
///
/// <https://dev.mysql.com/doc/internals/en/com-query.html>
/// <https://mariadb.com/kb/en/com_query/>
///
#[derive(Debug)]
pub(crate) struct Query<'q> {
    pub(crate) sql: &'q str,
}

impl Serialize<'_> for Query<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(0x03);
        buf.extend_from_slice(self.sql.as_bytes());

        Ok(())
    }
}

impl Command for Query<'_> {}
