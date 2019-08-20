use super::Decode;
use bytes::Bytes;
use std::{io, str};

// FIXME: Use &str functions for a custom Debug
#[derive(Debug)]
pub struct ParameterStatus {
    name: Vec<u8>,
    value: Vec<u8>,
}

impl ParameterStatus {
    #[inline]
    pub fn name(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.name) }
    }

    #[inline]
    pub fn value(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.value) }
    }
}

impl Decode for ParameterStatus {
    fn decode(src: &[u8]) -> io::Result<Self> {
        let name_end = memchr::memchr(0, &src).unwrap();
        let value_end = memchr::memchr(0, &src[(name_end + 1)..]).unwrap();

        let name = src[..name_end].into();
        let value = src[(name_end + 1)..(name_end + 1 + value_end)].into();

        Ok(Self { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ParameterStatus};
    use bytes::Bytes;
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
