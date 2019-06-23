use crate::Decode;
use bytes::{Buf, Bytes};
use std::io::{self, Cursor};

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
        let mut reader = Cursor::new(src);
        let process_id = reader.get_u32_be();
        let secret_key = reader.get_u32_be();

        Ok(Self { process_id, secret_key })
    }
}
