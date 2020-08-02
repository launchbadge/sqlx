use sqlx_core::error::Error;
use sqlx_core::io::Encode;

/// The normal, graceful termination procedure is that the frontend
/// sends a Terminate message and immediately closes the connection.
///
/// On receipt of this message, the backend closes the connection
/// and terminates.
///
#[derive(Debug)]
pub(crate) struct Terminate;

impl Encode<'_> for Terminate {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        buf.reserve(5);
        buf.push(b'X');
        buf.extend(&4_u32.to_be_bytes());

        Ok(())
    }
}
