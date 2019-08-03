use super::{BufMut, Encode};

/// Sent instead of [`StartupMessage`] with a new connection to cancel a running query on an existing
/// connection.
///
/// https://www.postgresql.org/docs/devel/protocol-flow.html#id-1.10.5.7.9
pub struct CancelRequest {
    /// The process ID of the target backend.
    pub process_id: i32,

    /// The secret key for the target backend.
    pub secret_key: i32,
}

impl Encode for CancelRequest {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_int_32(16); // message length
        buf.put_int_32(8087_7102); // constant for cancel request
        buf.put_int_32(self.process_id);
        buf.put_int_32(self.secret_key);
    }
}
