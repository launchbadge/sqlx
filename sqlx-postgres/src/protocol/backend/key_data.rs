use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct KeyData {
    /// The process ID of this database.
    pub(crate) process_id: u32,

    /// The secret key of this database.
    pub(crate) secret_key: u32,
}

impl Deserialize<'_> for KeyData {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let process_id = buf.get_u32();
        let secret_key = buf.get_u32();

        Ok(Self { process_id, secret_key })
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use sqlx_core::io::Deserialize;

    use super::KeyData;

    #[test]
    fn should_deserialize() {
        let m = KeyData::deserialize(Bytes::from_static(b"\0\0'\xc6\x89R\xc5+")).unwrap();

        assert_eq!(m.process_id, 10182);
        assert_eq!(m.secret_key, 2303903019);
    }
}
