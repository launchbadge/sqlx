#[cfg(test)]
#[doc(hidden)]
#[macro_export]
macro_rules! __bytes_builder (
    ($($b: expr), *) => {{
        use bytes::Buf;
        use bytes::IntoBuf;
        use bytes::BufMut;

        let mut bytes = bytes::BytesMut::new();
        $(
            {
                let buf = $b.into_buf();
                bytes.reserve(buf.remaining());
                bytes.put(buf);
            }
        )*
        bytes.freeze()
    }}
);

#[cfg(any(feature = "postgres"))]
macro_rules! invalid_data(
    ($($args:tt)*) => {
        $crate::error::InvalidData { args: format_args!($($args)*) }
    }
);
