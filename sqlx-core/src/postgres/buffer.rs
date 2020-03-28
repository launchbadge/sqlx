use crate::postgres::type_info::SharedStr;
use crate::postgres::PgConnection;
use byteorder::{ByteOrder, NetworkEndian};
use core::ops::{Deref, DerefMut};

#[derive(Debug, Default, PartialEq)]
pub struct PgRawBuffer {
    inner: Vec<u8>,

    // Whenever an `Encode` impl encounters a `PgTypeInfo` object that does not have an OID
    // It pushes a "hole" that must be patched later
    // The hole is a `usize` offset into the buffer with the type name that should be resolved
    // This is done for Records and Arrays as the OID is needed well before we are in an async
    // function and can just ask postgres
    type_holes: Vec<(usize, SharedStr)>,
}

impl PgRawBuffer {
    // Extends the inner buffer by enough space to have an OID
    // Remembers where the OID goes and type name for the OID
    pub(crate) fn push_type_hole(&mut self, type_name: &SharedStr) {
        let offset = self.len();

        self.extend_from_slice(&0_u32.to_be_bytes());
        self.type_holes.push((offset, type_name.clone()));
    }

    // Patch all remembered type holes
    // This should only go out and ask postgres if we have not seen the type name yet
    pub(crate) async fn patch_type_holes(
        &mut self,
        connection: &mut PgConnection,
    ) -> crate::Result<()> {
        for (offset, name) in &self.type_holes {
            let oid = connection.get_type_id_by_name(&*name).await?;
            NetworkEndian::write_u32(&mut self.inner[*offset..], oid);
        }

        Ok(())
    }
}

impl Deref for PgRawBuffer {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PgRawBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
