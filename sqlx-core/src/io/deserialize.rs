use bytes::Bytes;

pub trait Deserialize<'de, Cx = ()>: Sized {
    #[inline]
    fn deserialize(buf: Bytes) -> crate::Result<Self>
    where
        Self: Deserialize<'de, ()>,
    {
        Self::deserialize_with(buf, ())
    }

    fn deserialize_with(buf: Bytes, context: Cx) -> crate::Result<Self>;
}
