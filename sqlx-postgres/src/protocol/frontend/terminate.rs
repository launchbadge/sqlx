use sqlx_core::io::Serialize;
use sqlx_core::Result;

/// On receipt of this message, the backend closes the connection
/// and terminates.
#[derive(Debug)]
pub(crate) struct Terminate;

impl Serialize<'_> for Terminate {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'X');
        buf.extend_from_slice(&4_i32.to_be_bytes());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx_core::io::Serialize;

    use super::Terminate;

    #[test]
    fn should_serialize() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        Terminate.serialize(&mut buf)?;

        assert_eq!(&buf, &[b'X']);

        Ok(())
    }
}
