use super::decode::{Decode, get_str};
use std::pin::Pin;
use std::ptr::NonNull;
use std::{io, str};

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
    fn decode(src: &[u8]) -> io::Result<Self> {
        let storage = Pin::new(Vec::from(src));

        let name = get_str(&storage).unwrap();
        let value = get_str(&storage[name.len() + 1..]).unwrap();

        let name = NonNull::from(name);
        let value = NonNull::from(value);

        Ok(Self { storage, name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ParameterStatus};
    use std::io;

    const PARAM_STATUS: &[u8] = b"session_authorization\0postgres\0";

    #[test]
    fn it_decodes_param_status() -> io::Result<()> {
        let message = ParameterStatus::decode(PARAM_STATUS)?;

        assert_eq!(message.name(), "session_authorization");
        assert_eq!(message.value(), "postgres");

        Ok(())
    }
}
