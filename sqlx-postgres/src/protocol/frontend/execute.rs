use crate::io::PgWriteExt;
use crate::protocol::frontend::PortalRef;
use sqlx_core::io::Serialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct Execute {
    pub(crate) portal: PortalRef,

    /// Maximum number of rows to return, if portal contains a query
    /// that returns rows (ignored otherwise). Zero denotes “no limit”.
    pub(crate) max_rows: u32,
}

impl Serialize<'_> for Execute {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'E');
        buf.write_len_prefixed(|buf| {
            self.portal.serialize(buf)?;
            buf.extend(&self.max_rows.to_be_bytes());

            Ok(())
        })
    }
}
