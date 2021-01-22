use sqlx_core::io::Serialize;
use sqlx_core::Result;

#[derive(Debug)]
pub struct Terminate;

impl Serialize<'_, ()> for Terminate {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'X');
        buf.extend(&4_u32.to_be_bytes());

        Ok(())
    }
}
