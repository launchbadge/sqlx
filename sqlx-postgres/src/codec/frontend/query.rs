use crate::io::put_str;
use sqlx_core::error::Error;
use sqlx_core::io::Encode;

/// A simple query cycle is initiated by the frontend sending a `Query` message to the backend.
/// The message includes an SQL command (or commands) expressed as a text string.
#[derive(Debug)]
pub(crate) struct Query<'a>(pub(crate) &'a str);

impl Encode<'_> for Query<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        let len = 4 + self.0.len() + 1;

        if len + 1 > i32::MAX as usize {
            return Err(Error::Query(
                "SQL query string is too large to transmit".into(),
            ));
        }

        buf.reserve(len + 1);
        buf.push(b'Q');
        buf.extend(&(len as i32).to_be_bytes());
        put_str(buf, self.0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        const EXPECTED: &[u8] = b"Q\0\0\0\rSELECT 1\0";

        let mut buf = Vec::new();

        Query("SELECT 1").encode(&mut buf);

        assert_eq!(buf, EXPECTED);
    }
}
