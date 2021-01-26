pub trait Serialize<'ser, Cx = ()>: Sized {
    #[inline]
    fn serialize(&self, buf: &mut Vec<u8>) -> crate::Result<()>
    where
        Self: Serialize<'ser, ()>,
    {
        self.serialize_with(buf, ())
    }

    fn serialize_with(&self, buf: &mut Vec<u8>, context: Cx) -> crate::Result<()>;
}

impl<Cx> Serialize<'_, Cx> for &'_ [u8] {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: Cx) -> crate::Result<()> {
        buf.extend_from_slice(*self);

        Ok(())
    }
}
