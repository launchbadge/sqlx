use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::protocol::frontend::Target;

#[derive(Debug)]
pub(crate) struct Describe {
    pub(crate) target: Target,
}

impl Serialize<'_> for Describe {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'D');
        self.target.serialize(buf)
    }
}
