use super::decode::{Buf, Decode};
use std::{io, pin::Pin, ptr::NonNull, str};

// FIXME: Use &str functions for a custom Debug
#[derive(Debug)]
pub struct ParameterStatus {
    #[used]
    storage: Pin<Box<[u8]>>,
    name: NonNull<str>,
    value: NonNull<str>,
}

// SAFE: Raw pointers point to pinned memory inside the struct
unsafe impl Send for ParameterStatus {}
unsafe impl Sync for ParameterStatus {}

impl ParameterStatus {
    #[inline]
    pub fn name(&self) -> &str {
        // SAFE: Memory is pinned
        unsafe { self.name.as_ref() }
    }

    #[inline]
    pub fn value(&self) -> &str {
        // SAFE: Memory is pinned
        unsafe { self.value.as_ref() }
    }
}

impl Decode for ParameterStatus {
    fn decode(src: &[u8]) -> io::Result<Self> {
        let storage = Pin::new(src.into());
        let mut src: &[u8] = &*storage;

        let name = NonNull::from(src.get_str_null()?);
        let value = NonNull::from(src.get_str_null()?);

        Ok(Self {
            storage,
            name,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ParameterStatus};

    const PARAM_STATUS: &[u8] = b"session_authorization\0postgres\0";

    #[test]
    fn it_decodes_param_status() {
        let message = ParameterStatus::decode(PARAM_STATUS).unwrap();

        assert_eq!(message.name(), "session_authorization");
        assert_eq!(message.value(), "postgres");
    }

    #[bench]
    fn bench_decode_param_status(b: &mut test::Bencher) {
        b.iter(|| ParameterStatus::decode(PARAM_STATUS).unwrap());
    }
}
