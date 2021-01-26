use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::protocol::Command;

/// Tells the server that the client wants to close the connection.
///
/// https://dev.mysql.com/doc/internals/en/com-quit.html
///
#[derive(Debug)]
pub(crate) struct Quit;

impl Serialize<'_> for Quit {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(0x01);

        Ok(())
    }
}

impl Command for Quit {}

#[cfg(test)]
mod tests {
    use sqlx_core::io::Serialize;

    use super::Quit;

    #[test]
    fn should_serialize() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        Quit.serialize(&mut buf)?;

        assert_eq!(&buf, &[0x01]);

        Ok(())
    }
}
