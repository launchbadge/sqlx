use bytes::Bytes;

#[derive(Debug)]
pub struct BackendKeyData {
    /// The process ID of this backend.
    pub process_id: u32,

    /// The secret key of this backend.
    pub secret_key: u32,
}
