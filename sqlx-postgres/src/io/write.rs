use sqlx_core::io::WriteExt;
use sqlx_core::Result;

pub trait PgWriteExt: WriteExt {
    fn write_len_prefixed<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<()>;
}

impl PgWriteExt for Vec<u8> {
    /// Writes a length-prefixed message, this is used when encoding nearly
    /// all messages as postgres wants us to send the length of the
    /// often-variable-sized messages up front.
    fn write_len_prefixed<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<()>,
    {
        // reserve space to write the prefixed length
        let offset = self.len();
        self.extend_from_slice(&[0; 4]);

        // write the main body of the message
        f(self)?;

        // now calculate the size of what we wrote and set the length value
        let size = (self.len() - offset) as i32;
        self[offset..(offset + 4)].copy_from_slice(&size.to_be_bytes());

        Ok(())
    }
}
