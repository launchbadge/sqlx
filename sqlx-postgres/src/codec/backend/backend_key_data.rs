use bytes::{Buf, Bytes};
use sqlx_core::{error::Error, io::Decode};

/// Contains cancellation key data. The frontend must save these values if it
/// wishes to be able to issue `CancelRequest` messages later.
#[derive(Debug)]
pub(crate) struct BackendKeyData {
    /// The process ID of this database.
    pub(crate) process_id: u32,

    /// The secret key of this database.
    pub(crate) secret_key: u32,
}

impl Decode<'_> for BackendKeyData {
    fn decode_with(mut buf: Bytes, _: ()) -> Result<Self, Error> {
        let process_id = buf.get_u32();
        let secret_key = buf.get_u32();

        Ok(Self {
            process_id,
            secret_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        const DATA: &[u8] = b"\0\0'\xc6\x89R\xc5+";

        let m = BackendKeyData::decode(DATA.into()).unwrap();

        assert_eq!(m.process_id, 10182);
        assert_eq!(m.secret_key, 2303903019);
    }
}
