use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::protocol::Capabilities;

/// Check if the server is alive.
///
/// https://dev.mysql.com/doc/internals/en/com-ping.html
/// https://mariadb.com/kb/en/com_ping/
///
pub(crate) struct Ping;

impl Serialize<'_, Capabilities> for Ping {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: Capabilities) -> Result<()> {
        buf.push(0x0e);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx_core::io::Serialize;

    use super::Ping;
    use crate::protocol::Capabilities;

    #[test]
    fn should_serialize() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        Ping.serialize_with(&mut buf, Capabilities::empty())?;

        assert_eq!(&buf, &[0x0e]);

        Ok(())
    }
}
