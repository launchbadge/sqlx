use sqlx_core::io::Serialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) enum PortalRef {
    Unnamed,
    Named(u32),
}

impl Serialize<'_> for PortalRef {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        if let PortalRef::Named(id) = self {
            buf.extend_from_slice(b"_sqlx_p_");

            itoa::write(&mut *buf, *id).unwrap();
        }

        buf.push(b'\0');

        Ok(())
    }
}
