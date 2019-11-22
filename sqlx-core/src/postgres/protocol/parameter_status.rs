use super::decode::Decode;
use crate::io::Buf;
use std::{
    fmt::{self, Debug},
    io,
    pin::Pin,
    ptr::NonNull,
    str,
};

pub struct ParameterStatus {
    #[used]
    buffer: Pin<Box<[u8]>>,
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
    fn decode(buf: &[u8]) -> crate::Result<Self> {
        let buffer = Pin::new(buf.into());
        let mut buf: &[u8] = &*buffer;

        let name = buf.get_str_nul()?.into();
        let value = buf.get_str_nul()?.into();

        Ok(Self {
            buffer,
            name,
            value,
        })
    }
}

impl Debug for ParameterStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParameterStatus")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
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

        assert_eq!(
            format!("{:?}", message),
            "ParameterStatus { name: \"session_authorization\", value: \"postgres\" }"
        );
    }
}
