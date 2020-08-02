use crate::error::Error;

/// An object that can be encoded to a byte buffer.
/// Context is optional metadata that is required to encode this object.
pub trait Encode<'en, Context = ()> {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), Error>
    where
        Self: Encode<'en, ()>,
    {
        self.encode_with(buf, ())
    }

    fn encode_with(&self, buf: &mut Vec<u8>, context: Context) -> Result<(), Error>;
}

impl<C> Encode<'_, C> for &'_ [u8] {
    #[inline]
    fn encode_with(&self, buf: &mut Vec<u8>, _: C) -> Result<(), Error> {
        buf.extend_from_slice(self);

        Ok(())
    }
}
