use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::protocol::frontend::Target;

#[derive(Debug)]
pub(crate) struct Close {
    target: Target,
}

impl Serialize<'_> for Close {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'C');
        self.target.serialize(buf)
    }
}
