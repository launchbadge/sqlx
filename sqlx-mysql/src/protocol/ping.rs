use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::protocol::Command;

/// Check if the server is alive.
///
/// https://dev.mysql.com/doc/internals/en/com-ping.html
/// https://mariadb.com/kb/en/com_ping/
///
#[derive(Debug)]
pub(crate) struct Ping;

impl Serialize<'_> for Ping {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(0x0e);

        Ok(())
    }
}

impl Command for Ping {}

#[cfg(test)]
mod tests {
    use sqlx_core::io::Serialize;

    use super::Ping;

    #[test]
    fn should_serialize() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        Ping.serialize(&mut buf)?;

        assert_eq!(&buf, &[0x0e]);

        Ok(())
    }
}
