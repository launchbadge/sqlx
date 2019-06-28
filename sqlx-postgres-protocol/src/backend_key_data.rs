use crate::Decode;
use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use std::io;

#[derive(Debug)]
pub struct BackendKeyData {
    /// The process ID of this backend.
    process_id: u32,

    /// The secret key of this backend.
    secret_key: u32,
}

impl BackendKeyData {
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn secret_key(&self) -> u32 {
        self.secret_key
    }
}

impl Decode for BackendKeyData {
    fn decode(src: Bytes) -> io::Result<Self> {
        let process_id = BigEndian::read_u32(&src[..4]);
        let secret_key = BigEndian::read_u32(&src[4..]);

        Ok(Self { process_id, secret_key })
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendKeyData};
    use crate::{Decode};
    use bytes::Bytes;
    use std::io;

    const BACKEND_KEY_DATA: &[u8] = b"\0\0'\xc6\x89R\xc5+";

    #[test]
    fn it_decodes_backend_key_data() -> io::Result<()> {
        let src = Bytes::from_static(BACKEND_KEY_DATA);
        let message = BackendKeyData::decode(src)?;

        assert_eq!(message.process_id(), 10182);
        assert_eq!(message.secret_key(), 2303903019);

        Ok(())
    }
}
