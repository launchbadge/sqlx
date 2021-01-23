use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Error;
use sqlx_core::Result;

/// Contains cancellation key data. The frontend must save these values if it
/// wishes to be able to issue `CancelRequest` messages later.
#[derive(Debug)]
pub struct BackendKeyData {
    /// The process ID of this database.
    pub process_id: u32,

    /// The secret key of this database.
    pub secret_key: u32,
}

impl Deserialize<'_, ()> for BackendKeyData {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let process_id = BigEndian::read_u32(&buf);
        let secret_key = BigEndian::read_u32(&buf[4..]);

        Ok(Self { process_id, secret_key })
    }
}
