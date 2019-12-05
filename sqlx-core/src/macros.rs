#[cfg(test)]
#[doc(hidden)]
#[macro_export]
macro_rules! __bytes_builder (
    ($($b: expr), *) => {{
        use $crate::io::ToBuf;

        let mut buf = Vec::new();
        $(
            buf.extend_from_slice($b.to_buf());
        )*
        buf
    }}
);
