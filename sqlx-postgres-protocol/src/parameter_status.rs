use crate::{decode::get_str_bytes_unchecked, Decode};
use bytes::Bytes;
use std::{io, str};

// FIXME: Use &str functions for a custom Debug
#[derive(Debug)]
pub struct ParameterStatus {
    name: Bytes,
    value: Bytes,
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
    fn decode(src: Bytes) -> io::Result<Self> {
        let name = get_str_bytes_unchecked(&src);
        let value = get_str_bytes_unchecked(&src.slice_from(name.len() + 1));

        Ok(Self { name, value })
    }
}
