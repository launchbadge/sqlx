use sqlx_core::io::Serialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct Sync;

impl Serialize<'_> for Sync {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'S');
        buf.extend_from_slice(&4_i32.to_be_bytes());

        Ok(())
    }
}
