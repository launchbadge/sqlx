use crate::io::PgWriteExt;
use crate::protocol::frontend::Target;
use sqlx_core::io::{Serialize, WriteExt};
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct Close {
    target: Target,
}

impl Serialize<'_> for Close {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'C');
        buf.write_len_prefixed(|buf| self.target.serialize(buf))
    }
}
