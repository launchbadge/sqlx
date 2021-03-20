use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::io::PgWriteExt;

#[derive(Debug)]
pub(crate) struct Query<'a>(pub(crate) &'a str);

impl Serialize<'_> for Query<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.reserve(1 + self.0.len() + 1 + 4);

        buf.push(b'Q');

        buf.write_len_prefixed(|buf| {
            buf.extend_from_slice(self.0.as_bytes());
            buf.push(0);

            Ok(())
        })
    }
}
