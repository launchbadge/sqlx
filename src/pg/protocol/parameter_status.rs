use super::decode::{get_str, Decode};
use std::{io, pin::Pin, ptr::NonNull, str};

// FIXME: Use &str functions for a custom Debug
#[derive(Debug)]
pub struct ParameterStatus {
    #[used]
    storage: Pin<Vec<u8>>,
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
    fn decode(src: &[u8]) -> Self {
        let storage = Pin::new(Vec::from(src));

        let name = get_str(&storage);
        let value = get_str(&storage[name.len() + 1..]);

        let name = NonNull::from(name);
        let value = NonNull::from(value);

        Self {
            storage,
            name,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ParameterStatus};

    const PARAM_STATUS: &[u8] = b"session_authorization\0postgres\0";

    #[test]
    fn it_decodes_param_status() {
        let message = ParameterStatus::decode(PARAM_STATUS);

        assert_eq!(message.name(), "session_authorization");
        assert_eq!(message.value(), "postgres");
    }
}
