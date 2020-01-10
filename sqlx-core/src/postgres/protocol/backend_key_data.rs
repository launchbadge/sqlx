use super::Decode;
use crate::io::Buf;
use byteorder::NetworkEndian;

#[derive(Debug)]
pub struct BackendKeyData {
    /// The process ID of this database.
    pub process_id: u32,

    /// The secret key of this database.
    pub secret_key: u32,
}

impl Decode for BackendKeyData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let process_id = buf.get_u32::<NetworkEndian>()?;
        let secret_key = buf.get_u32::<NetworkEndian>()?;

        Ok(Self {
            process_id,
            secret_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendKeyData, Decode};

    const BACKEND_KEY_DATA: &[u8] = b"\0\0'\xc6\x89R\xc5+";

    #[test]
    fn it_decodes_backend_key_data() {
        let message = BackendKeyData::decode(BACKEND_KEY_DATA).unwrap();

        assert_eq!(message.process_id, 10182);
        assert_eq!(message.secret_key, 2303903019);
    }
}
