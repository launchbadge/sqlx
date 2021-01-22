use sqlx_core::io::Serialize;
use sqlx_core::Result;

// The Flush message does not cause any specific output to be generated,
// but forces the backend to deliver any data pending in its output buffers.

// A Flush must be sent after any extended-query command except Sync, if the
// frontend wishes to examine the results of that command before issuing more commands.

#[derive(Debug)]
pub struct Flush;

impl Serialize<'_, ()> for Flush {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'H');
        buf.extend(&4_i32.to_be_bytes());
    }
}
