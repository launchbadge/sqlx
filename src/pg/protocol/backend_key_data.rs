use super::Decode;
use bytes::Bytes;
use std::{convert::TryInto, io};

#[derive(Debug)]
pub struct BackendKeyData {
    /// The process ID of this backend.
    process_id: u32,

    /// The secret key of this backend.
    secret_key: u32,
}

impl BackendKeyData {
    #[inline]
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    #[inline]
    pub fn secret_key(&self) -> u32 {
        self.secret_key
    }
}

impl BackendKeyData {
    pub fn decode2(src: &[u8]) -> Self {
        // todo: error handling
        assert_eq!(src.len(), 8);
        let process_id = u32::from_be_bytes(src[0..4].try_into().unwrap());
        let secret_key = u32::from_be_bytes(src[4..8].try_into().unwrap());

        Self {
            process_id,
            secret_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendKeyData, Decode};
    use bytes::Bytes;
    use std::io;

    const BACKEND_KEY_DATA: &[u8] = b"\0\0'\xc6\x89R\xc5+";

    #[test]
    fn it_decodes_backend_key_data() -> io::Result<()> {
        let src = BACKEND_KEY_DATA;
        let message = BackendKeyData::decode2(src);

        assert_eq!(message.process_id(), 10182);
        assert_eq!(message.secret_key(), 2303903019);

        Ok(())
    }
}
