use sqlx_core::io::{Serialize, WriteExt};
use sqlx_core::Result;

use crate::io::PgWriteExt;
use crate::protocol::frontend::{PortalRef, StatementRef};

#[derive(Debug)]
pub(crate) struct Parse<'a> {
    pub(crate) statement: StatementRef,
    pub(crate) sql: &'a str,

    /// The parameter data types specified (could be zero). Note that this is not an
    /// indication of the number of parameters that might appear in the query string,
    /// only the number that the frontend wants to pre-specify types for.
    pub(crate) parameters: &'a [u32],
}

impl Serialize<'_> for Parse<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'P');
        buf.write_len_prefixed(|buf| {
            self.statement.serialize(buf)?;

            buf.write_str_nul(self.sql);

            // TODO: return a proper error
            assert!(!(self.parameters.len() >= (u16::MAX as usize)));

            buf.extend(&(self.parameters.len() as u16).to_be_bytes());

            for &oid in self.parameters {
                buf.extend(&oid.to_be_bytes());
            }

            Ok(())
        })
    }
}
