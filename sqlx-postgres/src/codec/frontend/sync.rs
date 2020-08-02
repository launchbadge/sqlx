use sqlx_core::error::Error;
use sqlx_core::io::Encode;

/// At completion of each series of extended-query messages, the frontend should issue a
/// `Sync` message.
///
/// This parameterless message causes the backend to close the current transaction if
/// it's not inside a `BEGIN` / `COMMIT` transaction block (“close” meaning to commit
/// if no error, or roll back if error). Then a `ReadyForQuery` response is issued.
///
/// The purpose of Sync is to provide a resynchronization point for error recovery.
///
#[derive(Debug)]
pub(crate) struct Sync;

impl Encode<'_> for Sync {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        buf.reserve(5);
        buf.push(b'S');
        buf.extend(&4_i32.to_be_bytes());

        Ok(())
    }
}
