use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::protocol::{Capabilities, Command};

/// Tells the server that the client wants to close the connection.
///
/// https://dev.mysql.com/doc/internals/en/com-quit.html
///
#[derive(Debug)]
pub(crate) struct Quit;

impl Serialize<'_, Capabilities> for Quit {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: Capabilities) -> Result<()> {
        buf.push(0x01);

        Ok(())
    }
}

impl Command for Quit {}

#[cfg(test)]
mod tests {
    use sqlx_core::io::Serialize;

    use super::Quit;
    use crate::protocol::Capabilities;

    #[test]
    fn should_serialize() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        Quit.serialize_with(&mut buf, Capabilities::empty())?;

        assert_eq!(&buf, &[0x01]);

        Ok(())
    }
}
